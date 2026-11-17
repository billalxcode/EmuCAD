//! Digital CAN 2.0A data frame transmitter.

use crate::catalog_bits::{equals, increment, less_than, mux_tree, register_bits};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 78] = [
    "reset", "start", "ack", "id0", "id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8", "id9",
    "id10", "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7", "din8", "din9",
    "din10", "din11", "din12", "din13", "din14", "din15", "din16", "din17", "din18", "din19",
    "din20", "din21", "din22", "din23", "din24", "din25", "din26", "din27", "din28", "din29",
    "din30", "din31", "din32", "din33", "din34", "din35", "din36", "din37", "din38", "din39",
    "din40", "din41", "din42", "din43", "din44", "din45", "din46", "din47", "din48", "din49",
    "din50", "din51", "din52", "din53", "din54", "din55", "din56", "din57", "din58", "din59",
    "din60", "din61", "din62", "din63",
];
const OUTPUTS: [&str; 4] = ["tx", "busy", "done", "nack"];

const ID_NAMES: [&str; 11] = [
    "id0", "id1", "id2", "id3", "id4", "id5", "id6", "id7", "id8", "id9", "id10",
];
const DATA_NAMES: [&str; 64] = [
    "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7", "din8", "din9", "din10",
    "din11", "din12", "din13", "din14", "din15", "din16", "din17", "din18", "din19", "din20",
    "din21", "din22", "din23", "din24", "din25", "din26", "din27", "din28", "din29", "din30",
    "din31", "din32", "din33", "din34", "din35", "din36", "din37", "din38", "din39", "din40",
    "din41", "din42", "din43", "din44", "din45", "din46", "din47", "din48", "din49", "din50",
    "din51", "din52", "din53", "din54", "din55", "din56", "din57", "din58", "din59", "din60",
    "din61", "din62", "din63",
];

const POLY: u32 = 0x4599;
const DATA_END: u32 = 83;
const CRC_END: u32 = 98;
const ACK_SLOT: u32 = 99;
const LAST_BIT: u32 = 110;
const FRAME_END: u32 = 111;

#[derive(Clone, Copy)]
struct CanInputs {
    reset: NetId,
    start: NetId,
    ack: NetId,
    id: [NetId; 11],
    data: [NetId; 64],
}

#[derive(Clone, Copy)]
struct CanState {
    active: NetId,
    pos: [NetId; 7],
    stuff: NetId,
    last: NetId,
    run: [NetId; 3],
    crc: [NetId; 15],
    nack: NetId,
}

#[derive(Clone, Copy)]
struct CanFlags {
    start_load: NetId,
    normal: NetId,
    pos_lt_83: NetId,
    pos_lt_98: NetId,
    ack_slot: NetId,
    last_pos: NetId,
    done: NetId,
}

#[derive(Clone, Copy)]
struct CanSignals {
    start_load: NetId,
    normal: NetId,
    schedule: NetId,
    bit: NetId,
    same: NetId,
    pos_lt_83: NetId,
    pos_lt_98: NetId,
    ack_slot: NetId,
    last_pos: NetId,
    done: NetId,
}

pub(crate) fn can_entry() -> DigitalEntry {
    finish("mcu.serial.can", |builder| {
        let inputs = can_inputs(builder);
        let state = can_state(builder);
        can_registers(builder, inputs, state);
        can_outputs(builder, inputs, state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn can_inputs(builder: &mut DigitalBuilder) -> CanInputs {
    CanInputs {
        reset: builder.input("reset"),
        start: builder.input("start"),
        ack: builder.input("ack"),
        id: std::array::from_fn(|index| builder.input(ID_NAMES[index])),
        data: std::array::from_fn(|index| builder.input(DATA_NAMES[index])),
    }
}

fn can_state(builder: &mut DigitalBuilder) -> CanState {
    CanState {
        active: builder.net(),
        pos: std::array::from_fn(|_| builder.net()),
        stuff: builder.net(),
        last: builder.net(),
        run: [builder.net(), builder.net(), builder.net()],
        crc: std::array::from_fn(|_| builder.net()),
        nack: builder.net(),
    }
}

fn can_flags(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) -> CanFlags {
    let not_active = builder.not(state.active);
    let start_load = builder.and(inputs.start, not_active);
    let not_stuff = builder.not(state.stuff);
    let normal = builder.and(state.active, not_stuff);
    let pos_lt_83 = less_than(builder, &state.pos, DATA_END);
    let pos_lt_98 = less_than(builder, &state.pos, CRC_END);
    let ack_slot = equals(builder, &state.pos, ACK_SLOT);
    let last_pos = equals(builder, &state.pos, LAST_BIT);
    let done = can_done(builder, state, not_active);
    CanFlags {
        start_load,
        normal,
        pos_lt_83,
        pos_lt_98,
        ack_slot,
        last_pos,
        done,
    }
}

fn can_done(builder: &mut DigitalBuilder, state: CanState, not_active: NetId) -> NetId {
    let frame_end = equals(builder, &state.pos, FRAME_END);
    builder.and(not_active, frame_end)
}

fn can_signals(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) -> CanSignals {
    let flags = can_flags(builder, inputs, state);
    let bit = frame_bit(builder, inputs, state);
    let different = builder.xor(bit, state.last);
    let same = builder.not(different);
    let run_full = equals(builder, &state.run, 4);
    let schedule = can_schedule(builder, flags.normal, flags.pos_lt_98, run_full, same);
    CanSignals {
        start_load: flags.start_load,
        normal: flags.normal,
        schedule,
        bit,
        same,
        pos_lt_83: flags.pos_lt_83,
        pos_lt_98: flags.pos_lt_98,
        ack_slot: flags.ack_slot,
        last_pos: flags.last_pos,
        done: flags.done,
    }
}

fn can_schedule(
    builder: &mut DigitalBuilder,
    normal: NetId,
    pos_lt_98: NetId,
    run_full: NetId,
    same: NetId,
) -> NetId {
    let repeat = builder.and(run_full, same);
    let region = builder.and(normal, pos_lt_98);
    builder.and(region, repeat)
}

fn can_registers(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) {
    let signals = can_signals(builder, inputs, state);
    let clear = builder.or(inputs.reset, signals.start_load);
    let pos = pos_next(builder, state, signals, clear);
    let active = active_next(builder, state, signals, inputs.reset);
    let crc = crc_update(builder, state, signals, clear);
    let last = last_next(builder, state, signals, clear);
    let run = run_next(builder, state, signals, clear);
    let nack = nack_next(builder, inputs, state, signals, clear);
    let stuff = stuff_next(builder, signals, clear);
    let one = builder.constant(true);
    register_bits(builder, &state.pos, &pos, one);
    register_bits(builder, &state.crc, &crc, one);
    register_bits(builder, &state.run, &run, one);
    builder.dff_into(active, state.active);
    builder.dff_into(last, state.last);
    builder.dff_into(nack, state.nack);
    builder.dff_into(stuff, state.stuff);
}

fn pos_next(
    builder: &mut DigitalBuilder,
    state: CanState,
    signals: CanSignals,
    clear: NetId,
) -> [NetId; 7] {
    let incremented = increment(builder, &state.pos);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(state.pos[index], incremented[index], signals.normal);
        builder.mux(value, zero, clear)
    })
}

fn active_next(
    builder: &mut DigitalBuilder,
    state: CanState,
    signals: CanSignals,
    reset: NetId,
) -> NetId {
    let last = builder.and(signals.normal, signals.last_pos);
    let not_last = builder.not(last);
    let stopped = builder.and(state.active, not_last);
    let set = builder.or(signals.start_load, stopped);
    let not_reset = builder.not(reset);
    builder.and(set, not_reset)
}

fn crc_update(
    builder: &mut DigitalBuilder,
    state: CanState,
    signals: CanSignals,
    clear: NetId,
) -> [NetId; 15] {
    let data_phase = builder.and(signals.normal, signals.pos_lt_83);
    let next = crc_next(builder, &state.crc, signals.bit, data_phase);
    let zero = builder.constant(false);
    std::array::from_fn(|index| builder.mux(next[index], zero, clear))
}

fn crc_next(
    builder: &mut DigitalBuilder,
    crc: &[NetId; 15],
    bit: NetId,
    data_phase: NetId,
) -> [NetId; 15] {
    let shifted = shift_crc(builder, crc);
    let feedback = builder.xor(crc[14], bit);
    std::array::from_fn(|index| {
        let base = shifted[index];
        let tapped = if POLY & (1 << index) != 0 {
            builder.xor(base, feedback)
        } else {
            base
        };
        builder.mux(crc[index], tapped, data_phase)
    })
}

fn shift_crc(builder: &mut DigitalBuilder, crc: &[NetId; 15]) -> [NetId; 15] {
    let zero = builder.constant(false);
    std::array::from_fn(|index| if index == 0 { zero } else { crc[index - 1] })
}

fn last_next(
    builder: &mut DigitalBuilder,
    state: CanState,
    signals: CanSignals,
    clear: NetId,
) -> NetId {
    let not_last = builder.not(state.last);
    let value = builder.mux(signals.bit, not_last, state.stuff);
    let one = builder.constant(true);
    builder.mux(value, one, clear)
}

fn run_next(
    builder: &mut DigitalBuilder,
    state: CanState,
    signals: CanSignals,
    clear: NetId,
) -> [NetId; 3] {
    let incremented = increment(builder, &state.run);
    let one = run_one(builder);
    let zero = builder.constant(false);
    let outside = run_outside(builder, signals);
    std::array::from_fn(|index| {
        let value = builder.mux(one[index], incremented[index], signals.same);
        let value = builder.mux(value, zero, outside);
        let value = builder.mux(value, one[index], state.stuff);
        builder.mux(value, zero, clear)
    })
}

fn run_one(builder: &mut DigitalBuilder) -> [NetId; 3] {
    [
        builder.constant(true),
        builder.constant(false),
        builder.constant(false),
    ]
}

fn run_outside(builder: &mut DigitalBuilder, signals: CanSignals) -> NetId {
    let region = builder.and(signals.normal, signals.pos_lt_98);
    builder.not(region)
}

fn nack_next(
    builder: &mut DigitalBuilder,
    inputs: CanInputs,
    state: CanState,
    signals: CanSignals,
    clear: NetId,
) -> NetId {
    let not_ack = builder.not(inputs.ack);
    let sample = builder.and(signals.ack_slot, signals.normal);
    let failure = builder.and(sample, not_ack);
    let value = builder.or(state.nack, failure);
    let zero = builder.constant(false);
    builder.mux(value, zero, clear)
}

fn stuff_next(builder: &mut DigitalBuilder, signals: CanSignals, clear: NetId) -> NetId {
    let zero = builder.constant(false);
    builder.mux(signals.schedule, zero, clear)
}

fn can_outputs(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) {
    let signals = can_signals(builder, inputs, state);
    let not_last = builder.not(state.last);
    let data = builder.mux(signals.bit, not_last, state.stuff);
    let one = builder.constant(true);
    let not_active = builder.not(state.active);
    let tx = builder.mux(data, one, not_active);
    builder.output("tx", tx);
    builder.output("busy", state.active);
    builder.output("done", signals.done);
    builder.output("nack", state.nack);
}

fn frame_bit(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) -> NetId {
    let level = frame_level(builder, inputs, state);
    mux_tree(builder, level, &state.pos)
}

fn frame_level(builder: &mut DigitalBuilder, inputs: CanInputs, state: CanState) -> Vec<NetId> {
    let mut level = header_level(builder, inputs);
    level.extend(data_level(inputs));
    level.extend((0..15).rev().map(|index| state.crc[index]));
    while level.len() < 128 {
        level.push(builder.constant(true));
    }
    level
}

fn header_level(builder: &mut DigitalBuilder, inputs: CanInputs) -> Vec<NetId> {
    let mut level = Vec::with_capacity(19);
    level.push(builder.constant(false));
    for index in (0..11).rev() {
        level.push(inputs.id[index]);
    }
    for _ in 0..3 {
        level.push(builder.constant(false));
    }
    level.push(builder.constant(true));
    for _ in 0..3 {
        level.push(builder.constant(false));
    }
    level
}

fn data_level(inputs: CanInputs) -> Vec<NetId> {
    let mut level = Vec::with_capacity(64);
    for byte in 0..8 {
        for bit in (0..8).rev() {
            level.push(inputs.data[byte * 8 + bit]);
        }
    }
    level
}

#[cfg(test)]
mod tests {
    use super::can_entry;

    fn load(entry: &mut super::DigitalEntry, id: u16, data: [u8; 8]) {
        for index in 0..11 {
            let bit = id & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("id{index}"), bit)
                .expect("id");
        }
        for index in 0..64 {
            let byte = data[index / 8];
            let bit = byte & (1 << (index % 8)) != 0;
            entry
                .circuit
                .set_input(&format!("din{index}"), bit)
                .expect("data");
        }
    }

    fn collect(entry: &mut super::DigitalEntry) -> Vec<bool> {
        let mut bits = Vec::new();
        entry.circuit.set_input("start", true).expect("start");
        entry.circuit.tick().expect("start");
        entry.circuit.set_input("start", false).expect("start");
        bits.push(entry.circuit.output("tx") == Some(true));
        while entry.circuit.output("done") != Some(true) {
            entry.circuit.tick().expect("frame");
            if entry.circuit.output("done") != Some(true) {
                bits.push(entry.circuit.output("tx") == Some(true));
            }
        }
        bits
    }

    fn expected() -> Vec<bool> {
        let text = "0001001000110001000001010001001000100011001101000100010101010110011001110111100010001000010001101111111111111111";
        text.chars().map(|value| value == '1').collect()
    }

    #[test]
    fn can_transmits_standard_frame() {
        let mut entry = can_entry();
        let data = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        load(&mut entry, 0x123, data);
        entry.circuit.set_input("ack", true).expect("ack");
        let bits = collect(&mut entry);
        assert_eq!(bits, expected());
        assert_eq!(entry.circuit.output("nack"), Some(false));
    }

    #[test]
    fn can_reports_missing_ack() {
        let mut entry = can_entry();
        let data = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        load(&mut entry, 0x123, data);
        entry.circuit.set_input("ack", false).expect("ack");
        drop(collect(&mut entry));
        assert_eq!(entry.circuit.output("nack"), Some(true));
    }
}
