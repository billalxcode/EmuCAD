//! Digital I2C EEPROM with 128 bytes and inverted storage.

use crate::catalog_bits::{equals, increment, mux_tree, register_bits, shift_in};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::catalog_memory::{DATA_BITS, array_cells, array_read};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 6] = ["reset", "scl", "sda", "a0", "a1", "a2"];
const OUTPUTS: [&str; 2] = ["sda_low", "busy"];

const IDLE: u32 = 0;
const ADDRESS: u32 = 1;
const WORD: u32 = 2;
const WRITE: u32 = 3;
const READ: u32 = 4;
const READ_ACK: u32 = 5;

#[derive(Clone, Copy)]
struct EepromInputs {
    reset: NetId,
    scl: NetId,
    sda: NetId,
    a0: NetId,
    a1: NetId,
    a2: NetId,
}

struct EepromState {
    phase: [NetId; 3],
    count: [NetId; 4],
    shift: [NetId; 8],
    word: [NetId; 7],
    matched: NetId,
    prev_scl: NetId,
    prev_sda: NetId,
    cells: Vec<Vec<NetId>>,
}

#[derive(Clone, Copy)]
struct EepromPhase {
    reset: NetId,
    flags: [NetId; 6],
    start: NetId,
    stop: NetId,
    rising: NetId,
    last: NetId,
    ack: NetId,
}

impl EepromPhase {
    fn idle(&self) -> NetId {
        self.flags[IDLE as usize]
    }
    fn address(&self) -> NetId {
        self.flags[ADDRESS as usize]
    }
    fn word(&self) -> NetId {
        self.flags[WORD as usize]
    }
    fn write(&self) -> NetId {
        self.flags[WRITE as usize]
    }
    fn read(&self) -> NetId {
        self.flags[READ as usize]
    }
    fn read_ack(&self) -> NetId {
        self.flags[READ_ACK as usize]
    }
}

struct EepromNext {
    phase: [NetId; 3],
    count: [NetId; 4],
    shift: [NetId; 8],
    word: [NetId; 7],
    matched: NetId,
}

#[derive(Clone, Copy)]
struct PhaseEvents {
    address_done: NetId,
    word_done: NetId,
    write_done: NetId,
    read_done: NetId,
    ack_event: NetId,
    ack_read: NetId,
}

#[derive(Clone, Copy)]
struct PhaseContext<'a> {
    phase: &'a EepromPhase,
    hold: NetId,
    events: &'a PhaseEvents,
    ack_stop: NetId,
}

pub(crate) fn i2c_eeprom_entry() -> DigitalEntry {
    finish("memory.device.i2c_eeprom", |builder| {
        let inputs = eeprom_inputs(builder);
        let state = eeprom_state(builder);
        eeprom_registers(builder, inputs, &state);
        eeprom_outputs(builder, inputs, &state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn eeprom_inputs(builder: &mut DigitalBuilder) -> EepromInputs {
    EepromInputs {
        reset: builder.input("reset"),
        scl: builder.input("scl"),
        sda: builder.input("sda"),
        a0: builder.input("a0"),
        a1: builder.input("a1"),
        a2: builder.input("a2"),
    }
}

fn eeprom_state(builder: &mut DigitalBuilder) -> EepromState {
    EepromState {
        phase: [builder.net(), builder.net(), builder.net()],
        count: std::array::from_fn(|_| builder.net()),
        shift: std::array::from_fn(|_| builder.net()),
        word: std::array::from_fn(|_| builder.net()),
        matched: builder.net(),
        prev_scl: builder.net(),
        prev_sda: builder.net(),
        cells: array_cells(builder),
    }
}

fn eeprom_phase(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    state: &EepromState,
) -> EepromPhase {
    let (start, stop, rising) = phase_timing(builder, inputs, state);
    let last = equals(builder, &state.count, 7);
    let ack = equals(builder, &state.count, 8);
    let flags = std::array::from_fn(|index| equals(builder, &state.phase, index as u32));
    EepromPhase {
        reset: inputs.reset,
        flags,
        start,
        stop,
        rising,
        last,
        ack,
    }
}

fn phase_timing(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    state: &EepromState,
) -> (NetId, NetId, NetId) {
    let start = start_condition(builder, inputs, state);
    let stop = stop_condition(builder, inputs, state);
    let rising = rising_edge(builder, inputs, state);
    (start, stop, rising)
}

fn start_condition(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    state: &EepromState,
) -> NetId {
    let not_sda = builder.not(inputs.sda);
    let data = builder.and(state.prev_sda, not_sda);
    let clock = builder.and(state.prev_scl, inputs.scl);
    builder.and(clock, data)
}

fn stop_condition(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    state: &EepromState,
) -> NetId {
    let not_prev = builder.not(state.prev_sda);
    let data = builder.and(not_prev, inputs.sda);
    let clock = builder.and(state.prev_scl, inputs.scl);
    builder.and(clock, data)
}

fn rising_edge(builder: &mut DigitalBuilder, inputs: EepromInputs, state: &EepromState) -> NetId {
    let not_prev = builder.not(state.prev_scl);
    builder.and(not_prev, inputs.scl)
}

fn eeprom_registers(builder: &mut DigitalBuilder, inputs: EepromInputs, state: &EepromState) {
    let phase = eeprom_phase(builder, inputs, state);
    let shifted = shift_in(&state.shift, inputs.sda);
    let next = eeprom_next(builder, inputs, state, &phase, &shifted);
    eeprom_memory(builder, state, &phase, &shifted);
    eeprom_store(builder, state, next, inputs);
}

fn eeprom_next(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    state: &EepromState,
    phase: &EepromPhase,
    shifted: &[NetId; 8],
) -> EepromNext {
    let matched = address_match(builder, shifted, inputs);
    let (phase_value, events) =
        phase_next(builder, phase, state.phase, shifted[0], matched, inputs.sda);
    let count = count_next(builder, phase, state.count, matched, &events);
    let (shift, word, matched) = eeprom_data_next(builder, inputs, phase, state, shifted, &events);
    EepromNext {
        phase: phase_value,
        count,
        shift,
        word,
        matched,
    }
}

fn eeprom_data_next(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    phase: &EepromPhase,
    state: &EepromState,
    shifted: &[NetId; 8],
    events: &PhaseEvents,
) -> ([NetId; 8], [NetId; 7], NetId) {
    let shift = shift_next(builder, inputs, phase, &state.shift, shifted);
    let word = word_next(builder, phase, state.word, shifted, events);
    let matched = matched_next(builder, inputs, phase, state.matched, events);
    (shift, word, matched)
}

fn address_match(builder: &mut DigitalBuilder, bits: &[NetId; 8], inputs: EepromInputs) -> NetId {
    let upper_bits = [bits[4], bits[5], bits[6], bits[7]];
    let upper = equals(builder, &upper_bits, 0b1010);
    let a2 = same_bit(builder, bits[3], inputs.a2);
    let a1 = same_bit(builder, bits[2], inputs.a1);
    let a0 = same_bit(builder, bits[1], inputs.a0);
    let low = builder.and(a1, a0);
    let low = builder.and(a2, low);
    builder.and(upper, low)
}

fn same_bit(builder: &mut DigitalBuilder, left: NetId, right: NetId) -> NetId {
    let different = builder.xor(left, right);
    builder.not(different)
}

fn address_target(builder: &mut DigitalBuilder, rw: NetId, matched: NetId) -> [NetId; 3] {
    std::array::from_fn(|index| {
        let word = const_bit(builder, WORD, index);
        let read = const_bit(builder, READ, index);
        let idle = const_bit(builder, IDLE, index);
        let value = builder.mux(word, read, rw);
        builder.mux(idle, value, matched)
    })
}

fn phase_events(builder: &mut DigitalBuilder, phase: &EepromPhase, sda: NetId) -> PhaseEvents {
    let last_rising = builder.and(phase.last, phase.rising);
    let address_done = builder.and(phase.address(), last_rising);
    let word_done = builder.and(phase.word(), last_rising);
    let write_done = builder.and(phase.write(), last_rising);
    let read_done = builder.and(phase.read(), last_rising);
    let ack_rising = builder.and(phase.ack, phase.rising);
    let ack_event = builder.and(phase.read_ack(), ack_rising);
    let not_sda = builder.not(sda);
    let ack_read = builder.and(ack_event, not_sda);
    PhaseEvents {
        address_done,
        word_done,
        write_done,
        read_done,
        ack_event,
        ack_read,
    }
}

fn phase_next(
    builder: &mut DigitalBuilder,
    phase: &EepromPhase,
    current: [NetId; 3],
    rw: NetId,
    matched: NetId,
    sda: NetId,
) -> ([NetId; 3], PhaseEvents) {
    let events = phase_events(builder, phase, sda);
    let hold = phase_hold(builder, phase, &events);
    let ack_stop = builder.and(events.ack_event, sda);
    let context = PhaseContext {
        phase,
        hold,
        events: &events,
        ack_stop,
    };
    let value = phase_bits(builder, current, rw, matched, &context);
    (value, events)
}

fn phase_bits(
    builder: &mut DigitalBuilder,
    current: [NetId; 3],
    rw: NetId,
    matched: NetId,
    context: &PhaseContext,
) -> [NetId; 3] {
    let target = address_target(builder, rw, matched);
    std::array::from_fn(|index| phase_bit(builder, current[index], index, target[index], context))
}

fn phase_hold(builder: &mut DigitalBuilder, phase: &EepromPhase, events: &PhaseEvents) -> NetId {
    let mut active = builder.or(phase.reset, phase.start);
    active = builder.or(active, phase.stop);
    active = builder.or(active, events.address_done);
    active = builder.or(active, events.word_done);
    active = builder.or(active, events.write_done);
    active = builder.or(active, events.read_done);
    active = builder.or(active, events.ack_event);
    builder.not(active)
}

fn phase_bit(
    builder: &mut DigitalBuilder,
    current: NetId,
    index: usize,
    target: NetId,
    context: &PhaseContext,
) -> NetId {
    let mut value = builder.and(current, context.hold);
    let address = builder.and(target, context.events.address_done);
    value = builder.or(value, address);
    let write = builder.or(context.events.word_done, context.events.write_done);
    let stop_reset = builder.or(context.phase.stop, context.phase.reset);
    let idle = builder.or(context.ack_stop, stop_reset);
    value = phase_add(builder, value, ADDRESS, index, context.phase.start);
    value = phase_add(builder, value, WRITE, index, write);
    value = phase_add(builder, value, READ_ACK, index, context.events.read_done);
    value = phase_add(builder, value, READ, index, context.events.ack_read);
    value = phase_add(builder, value, IDLE, index, idle);
    value
}

fn phase_add(
    builder: &mut DigitalBuilder,
    value: NetId,
    code: u32,
    index: usize,
    select: NetId,
) -> NetId {
    let term = phase_const(builder, code, index, select);
    builder.or(value, term)
}

fn phase_const(builder: &mut DigitalBuilder, code: u32, index: usize, select: NetId) -> NetId {
    let bit = const_bit(builder, code, index);
    builder.and(bit, select)
}

fn const_bit(builder: &mut DigitalBuilder, value: u32, index: usize) -> NetId {
    builder.constant(value & (1 << index) != 0)
}

fn count_next(
    builder: &mut DigitalBuilder,
    phase: &EepromPhase,
    current: [NetId; 4],
    matched: NetId,
    events: &PhaseEvents,
) -> [NetId; 4] {
    let incremented = increment(builder, &current);
    let zero_cond = count_zero(builder, phase, matched, events);
    let eight_cond = count_eight(builder, matched, events);
    let zero = builder.constant(false);
    let eight: [NetId; 4] = std::array::from_fn(|index| const_bit(builder, 8, index));
    std::array::from_fn(|index| {
        let value = builder.mux(current[index], incremented[index], phase.rising);
        let value = builder.mux(value, eight[index], eight_cond);
        builder.mux(value, zero, zero_cond)
    })
}

fn count_eight(builder: &mut DigitalBuilder, matched: NetId, events: &PhaseEvents) -> NetId {
    let address = builder.and(events.address_done, matched);
    let done = builder.or(events.word_done, events.write_done);
    let done = builder.or(done, events.read_done);
    builder.or(address, done)
}

fn count_zero(
    builder: &mut DigitalBuilder,
    phase: &EepromPhase,
    matched: NetId,
    events: &PhaseEvents,
) -> NetId {
    let not_matched = builder.not(matched);
    let address_bad = builder.and(events.address_done, not_matched);
    let read = builder.or(phase.read(), phase.read_ack());
    let write_read = builder.or(phase.write(), read);
    let word_write = builder.or(phase.word(), write_read);
    let ack_state = builder.or(phase.address(), word_write);
    let ack_rising = builder.and(phase.ack, phase.rising);
    let ack_zero = builder.and(ack_rising, ack_state);
    let start_stop = builder.or(phase.start, phase.stop);
    let bad_ack = builder.or(address_bad, ack_zero);
    let start_bad = builder.or(start_stop, bad_ack);
    builder.or(phase.reset, start_bad)
}

fn shift_next(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    phase: &EepromPhase,
    current: &[NetId; 8],
    shifted: &[NetId; 8],
) -> [NetId; 8] {
    let word_write = builder.or(phase.word(), phase.write());
    let write_phase = builder.or(phase.address(), word_write);
    let not_ack = builder.not(phase.ack);
    let write_ack = builder.and(write_phase, not_ack);
    let loading = builder.and(phase.rising, write_ack);
    let clear = builder.or(inputs.reset, phase.start);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(current[index], shifted[index], loading);
        builder.mux(value, zero, clear)
    })
}

fn word_next(
    builder: &mut DigitalBuilder,
    phase: &EepromPhase,
    current: [NetId; 7],
    shifted: &[NetId; 8],
    events: &PhaseEvents,
) -> [NetId; 7] {
    let incremented = increment(builder, &current);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(current[index], shifted[index], events.word_done);
        let value = builder.mux(value, incremented[index], events.write_done);
        let value = builder.mux(value, incremented[index], events.ack_read);
        builder.mux(value, zero, phase.reset)
    })
}

fn matched_next(
    builder: &mut DigitalBuilder,
    inputs: EepromInputs,
    phase: &EepromPhase,
    current: NetId,
    events: &PhaseEvents,
) -> NetId {
    let zero = builder.constant(false);
    let value = builder.mux(current, phase.address(), events.address_done);
    let clear = builder.or(inputs.reset, phase.start);
    builder.mux(value, zero, clear)
}

fn eeprom_memory(
    builder: &mut DigitalBuilder,
    state: &EepromState,
    phase: &EepromPhase,
    shifted: &[NetId; 8],
) {
    let last_rising = builder.and(phase.last, phase.rising);
    let write = builder.and(phase.write(), last_rising);
    for (index, word) in state.cells.iter().enumerate() {
        let selected = equals(builder, &state.word, index as u32);
        let word_write = builder.and(write, selected);
        eeprom_word(builder, word, shifted, word_write);
    }
}

fn eeprom_word(builder: &mut DigitalBuilder, word: &[NetId], shifted: &[NetId; 8], write: NetId) {
    for bit in 0..DATA_BITS {
        let data = builder.not(shifted[bit]);
        let next = builder.mux(word[bit], data, write);
        builder.dff_into(next, word[bit]);
    }
}

fn eeprom_store(
    builder: &mut DigitalBuilder,
    state: &EepromState,
    next: EepromNext,
    inputs: EepromInputs,
) {
    let one = builder.constant(true);
    register_bits(builder, &state.phase, &next.phase, one);
    register_bits(builder, &state.count, &next.count, one);
    register_bits(builder, &state.shift, &next.shift, one);
    register_bits(builder, &state.word, &next.word, one);
    builder.dff_into(next.matched, state.matched);
    builder.dff_into(inputs.scl, state.prev_scl);
    builder.dff_into(inputs.sda, state.prev_sda);
}

fn eeprom_outputs(builder: &mut DigitalBuilder, inputs: EepromInputs, state: &EepromState) {
    let phase = eeprom_phase(builder, inputs, state);
    let data = data_low(builder, &phase, state);
    let ack = ack_low(builder, &phase, state);
    let low = builder.or(data, ack);
    let busy = builder.not(phase.idle());
    builder.output("sda_low", low);
    builder.output("busy", busy);
}

fn data_low(builder: &mut DigitalBuilder, phase: &EepromPhase, state: &EepromState) -> NetId {
    let cells = array_read(builder, &state.cells, &state.word);
    let mut level = Vec::with_capacity(8);
    for index in (0..8).rev() {
        level.push(builder.not(cells[index]));
    }
    let bit = mux_tree(builder, level, &state.count[..3]);
    let not_bit = builder.not(bit);
    let not_ack = builder.not(phase.ack);
    let read = builder.and(phase.read(), not_ack);
    builder.and(read, not_bit)
}

fn ack_low(builder: &mut DigitalBuilder, phase: &EepromPhase, state: &EepromState) -> NetId {
    let ack_matched = builder.and(phase.ack, state.matched);
    let address = builder.and(phase.address(), ack_matched);
    let word = builder.and(phase.word(), phase.ack);
    let write = builder.and(phase.write(), phase.ack);
    let read = builder.and(phase.read(), phase.ack);
    let low = builder.or(address, word);
    let low = builder.or(low, write);
    builder.or(low, read)
}

#[cfg(test)]
mod tests {
    use super::i2c_eeprom_entry;

    fn idle(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("reset", true).expect("reset");
        entry.circuit.tick().expect("reset");
        entry.circuit.set_input("reset", false).expect("reset");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.set_input("sda", true).expect("sda");
        entry.circuit.settle().expect("idle");
        entry.circuit.tick().expect("idle");
    }

    fn start(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.set_input("sda", true).expect("sda");
        entry.circuit.tick().expect("start setup");
        entry.circuit.set_input("sda", false).expect("sda");
        entry.circuit.tick().expect("start");
    }

    fn send_bit(entry: &mut super::DigitalEntry, value: bool) {
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.set_input("sda", value).expect("sda");
        entry.circuit.tick().expect("low");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.tick().expect("high");
    }

    fn send(entry: &mut super::DigitalEntry, value: u8, ack: bool) {
        for index in (0..8).rev() {
            send_bit(entry, value & (1 << index) != 0);
        }
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.set_input("sda", true).expect("sda");
        entry.circuit.tick().expect("ack low");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.settle().expect("ack high");
        if ack {
            assert_eq!(entry.circuit.output("sda_low"), Some(true));
        }
        entry.circuit.tick().expect("ack");
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.tick().expect("ack low");
    }

    fn read_bit(entry: &mut super::DigitalEntry) -> bool {
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.tick().expect("low");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.settle().expect("high");
        let value = entry.circuit.output("sda_low") != Some(true);
        entry.circuit.tick().expect("high");
        value
    }

    fn read(entry: &mut super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..8 {
            if read_bit(entry) {
                value |= 1 << (7 - index);
            }
        }
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.set_input("sda", true).expect("sda");
        entry.circuit.tick().expect("nack low");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.tick().expect("nack");
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.tick().expect("nack low");
        value
    }

    fn stop(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("scl", false).expect("scl");
        entry.circuit.set_input("sda", false).expect("sda");
        entry.circuit.tick().expect("stop low");
        entry.circuit.set_input("scl", true).expect("scl");
        entry.circuit.tick().expect("stop high");
        entry.circuit.set_input("sda", true).expect("sda");
        entry.circuit.tick().expect("stop");
    }

    #[test]
    fn eeprom_writes_and_reads_byte() {
        let mut entry = i2c_eeprom_entry();
        idle(&mut entry);
        start(&mut entry);
        send(&mut entry, 0xa0, true);
        send(&mut entry, 0x05, true);
        send(&mut entry, 0x5a, true);
        stop(&mut entry);
        start(&mut entry);
        send(&mut entry, 0xa0, true);
        send(&mut entry, 0x05, true);
        start(&mut entry);
        send(&mut entry, 0xa1, true);
        assert_eq!(read(&mut entry), 0x5a);
        stop(&mut entry);
    }

    #[test]
    fn eeprom_ignores_wrong_address() {
        let mut entry = i2c_eeprom_entry();
        idle(&mut entry);
        start(&mut entry);
        send(&mut entry, 0xa8, false);
        assert_eq!(entry.circuit.output("busy"), Some(false));
    }
}
