//! Digital I2C master with open-drain bus outputs.

use crate::catalog_bits::{equals, increment, mux_tree, register_bits};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 19] = [
    "reset", "start", "rw", "ack", "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6",
    "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7",
];
const OUTPUTS: [&str; 5] = ["scl", "sda", "busy", "done", "nack"];

const IDLE: u32 = 0;
const START: u32 = 1;
const BIT_LOW: u32 = 2;
const BIT_HIGH: u32 = 3;
const STOP_LOW: u32 = 4;
const STOP_HIGH: u32 = 5;
const STOP_RELEASE: u32 = 6;
const DONE: u32 = 7;
const PHASE_NEXT: [u32; 8] = [
    IDLE,
    BIT_LOW,
    BIT_HIGH,
    BIT_LOW,
    STOP_HIGH,
    STOP_RELEASE,
    DONE,
    DONE,
];

#[derive(Clone, Copy)]
struct I2cInputs {
    reset: NetId,
    start: NetId,
    rw: NetId,
    ack: NetId,
    address: [NetId; 7],
    data: [NetId; 8],
}

#[derive(Clone, Copy)]
struct I2cState {
    phase: [NetId; 3],
    bit: [NetId; 5],
    nack: NetId,
}

#[derive(Clone, Copy)]
struct I2cPhases {
    idle: NetId,
    start: NetId,
    bit_low: NetId,
    bit_high: NetId,
    stop_low: NetId,
    stop_high: NetId,
    done: NetId,
}

pub(crate) fn i2c_entry() -> DigitalEntry {
    finish("mcu.serial.i2c", |builder| {
        let inputs = i2c_inputs(builder);
        let state = i2c_state(builder);
        i2c_registers(builder, inputs, state);
        i2c_outputs(builder, inputs, state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn i2c_inputs(builder: &mut DigitalBuilder) -> I2cInputs {
    I2cInputs {
        reset: builder.input("reset"),
        start: builder.input("start"),
        rw: builder.input("rw"),
        ack: builder.input("ack"),
        address: address_inputs(builder),
        data: data_inputs(builder),
    }
}

fn address_inputs(builder: &mut DigitalBuilder) -> [NetId; 7] {
    [
        builder.input("addr0"),
        builder.input("addr1"),
        builder.input("addr2"),
        builder.input("addr3"),
        builder.input("addr4"),
        builder.input("addr5"),
        builder.input("addr6"),
    ]
}

fn data_inputs(builder: &mut DigitalBuilder) -> [NetId; 8] {
    [
        builder.input("din0"),
        builder.input("din1"),
        builder.input("din2"),
        builder.input("din3"),
        builder.input("din4"),
        builder.input("din5"),
        builder.input("din6"),
        builder.input("din7"),
    ]
}

fn i2c_state(builder: &mut DigitalBuilder) -> I2cState {
    I2cState {
        phase: [builder.net(), builder.net(), builder.net()],
        bit: [
            builder.net(),
            builder.net(),
            builder.net(),
            builder.net(),
            builder.net(),
        ],
        nack: builder.net(),
    }
}

fn i2c_phases(builder: &mut DigitalBuilder, state: I2cState) -> I2cPhases {
    I2cPhases {
        idle: equals(builder, &state.phase, IDLE),
        start: equals(builder, &state.phase, START),
        bit_low: equals(builder, &state.phase, BIT_LOW),
        bit_high: equals(builder, &state.phase, BIT_HIGH),
        stop_low: equals(builder, &state.phase, STOP_LOW),
        stop_high: equals(builder, &state.phase, STOP_HIGH),
        done: equals(builder, &state.phase, DONE),
    }
}

fn i2c_registers(builder: &mut DigitalBuilder, inputs: I2cInputs, state: I2cState) {
    let phases = i2c_phases(builder, state);
    let last = equals(builder, &state.bit, 17);
    let phase = phase_next(builder, &state.phase, inputs.start, last);
    let bit = bit_next(builder, &phases, state.bit, last);
    let nack = nack_next(builder, inputs, &phases, state);
    register_all(builder, inputs.reset, state, phase, bit, nack);
}

fn phase_next(
    builder: &mut DigitalBuilder,
    phase: &[NetId; 3],
    start: NetId,
    last: NetId,
) -> [NetId; 3] {
    std::array::from_fn(|index| {
        let mut level: Vec<NetId> = PHASE_NEXT
            .iter()
            .map(|value| builder.constant(value & (1 << index) != 0))
            .collect();
        let start_bit = builder.constant(START & (1 << index) != 0);
        level[0] = builder.mux(level[0], start_bit, start);
        let stop_bit = builder.constant(STOP_LOW & (1 << index) != 0);
        level[3] = builder.mux(level[3], stop_bit, last);
        mux_tree(builder, level, phase)
    })
}

fn bit_next(
    builder: &mut DigitalBuilder,
    phases: &I2cPhases,
    bit: [NetId; 5],
    last: NetId,
) -> [NetId; 5] {
    let incremented = increment(builder, &bit);
    let zero = builder.constant(false);
    let last_high = builder.and(phases.bit_high, last);
    let clear = builder.or(phases.start, last_high);
    std::array::from_fn(|index| {
        let value = builder.mux(bit[index], incremented[index], phases.bit_high);
        builder.mux(value, zero, clear)
    })
}

fn nack_next(
    builder: &mut DigitalBuilder,
    inputs: I2cInputs,
    phases: &I2cPhases,
    state: I2cState,
) -> NetId {
    let first = equals(builder, &state.bit, 8);
    let second = equals(builder, &state.bit, 17);
    let ack_bit = builder.or(first, second);
    let sampling = builder.and(phases.bit_high, ack_bit);
    let not_ack = builder.not(inputs.ack);
    let failure = builder.and(sampling, not_ack);
    let not_start = builder.not(inputs.start);
    let keep = builder.and(state.nack, not_start);
    builder.or(keep, failure)
}

fn register_all(
    builder: &mut DigitalBuilder,
    reset: NetId,
    state: I2cState,
    phase: [NetId; 3],
    bit: [NetId; 5],
    nack: NetId,
) {
    let not_reset = builder.not(reset);
    register_bits(builder, &state.phase, &phase, not_reset);
    register_bits(builder, &state.bit, &bit, not_reset);
    let next = builder.and(nack, not_reset);
    builder.dff_into(next, state.nack);
}

fn i2c_outputs(builder: &mut DigitalBuilder, inputs: I2cInputs, state: I2cState) {
    let phases = i2c_phases(builder, state);
    let (scl, sda) = i2c_lines(builder, inputs, state, &phases);
    let inactive = builder.or(phases.idle, phases.done);
    let busy = builder.not(inactive);
    builder.output("scl", scl);
    builder.output("sda", sda);
    builder.output("busy", busy);
    builder.output("done", phases.done);
    builder.output("nack", state.nack);
}

fn i2c_lines(
    builder: &mut DigitalBuilder,
    inputs: I2cInputs,
    state: I2cState,
    phases: &I2cPhases,
) -> (NetId, NetId) {
    let scl = scl_line(builder, phases);
    let sda = sda_line(builder, inputs, state, phases);
    (scl, sda)
}

fn scl_line(builder: &mut DigitalBuilder, phases: &I2cPhases) -> NetId {
    let low = builder.or(phases.bit_low, phases.stop_low);
    builder.not(low)
}

fn sda_line(
    builder: &mut DigitalBuilder,
    inputs: I2cInputs,
    state: I2cState,
    phases: &I2cPhases,
) -> NetId {
    let own_low = sda_low(builder, inputs, state, phases);
    let ack_pull = ack_pull(builder, inputs, state, phases);
    let not_ack = builder.not(ack_pull);
    let released = builder.not(own_low);
    builder.and(released, not_ack)
}

fn sda_low(
    builder: &mut DigitalBuilder,
    inputs: I2cInputs,
    state: I2cState,
    phases: &I2cPhases,
) -> NetId {
    let value = bit_value(builder, inputs, state);
    let not_value = builder.not(value);
    let data_phase = builder.or(phases.bit_low, phases.bit_high);
    let data_low = builder.and(data_phase, not_value);
    let stop_low = builder.or(phases.stop_low, phases.stop_high);
    let low = builder.or(data_low, stop_low);
    builder.or(phases.start, low)
}

fn ack_pull(
    builder: &mut DigitalBuilder,
    inputs: I2cInputs,
    state: I2cState,
    phases: &I2cPhases,
) -> NetId {
    let first = equals(builder, &state.bit, 8);
    let second = equals(builder, &state.bit, 17);
    let ack_bit = builder.or(first, second);
    let slave = builder.and(ack_bit, inputs.ack);
    let data_phase = builder.or(phases.bit_low, phases.bit_high);
    builder.and(data_phase, slave)
}

fn bit_value(builder: &mut DigitalBuilder, inputs: I2cInputs, state: I2cState) -> NetId {
    let mut level = Vec::with_capacity(32);
    for index in 0..7 {
        level.push(inputs.address[6 - index]);
    }
    level.push(inputs.rw);
    level.push(builder.constant(true));
    for index in 0..8 {
        level.push(inputs.data[7 - index]);
    }
    level.push(builder.constant(true));
    while level.len() < 32 {
        level.push(builder.constant(true));
    }
    mux_tree(builder, level, &state.bit)
}

#[cfg(test)]
mod tests {
    use super::i2c_entry;

    fn load(entry: &mut super::DigitalEntry, address: u8, data: u8) {
        for index in 0..7 {
            let bit = address & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("addr{index}"), bit)
                .expect("addr");
        }
        for index in 0..8 {
            let bit = data & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("din{index}"), bit)
                .expect("din");
        }
    }

    fn start_transfer(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("start", true).expect("start");
        entry.circuit.tick().expect("start");
        assert_eq!(entry.circuit.output("scl"), Some(true));
        assert_eq!(entry.circuit.output("sda"), Some(false));
        assert_eq!(entry.circuit.output("busy"), Some(true));
        entry.circuit.set_input("start", false).expect("start");
        entry.circuit.tick().expect("bit low");
    }

    fn collect_bits(entry: &mut super::DigitalEntry) -> Vec<bool> {
        let mut bits = Vec::with_capacity(18);
        for bit in 0..18 {
            entry.circuit.tick().expect("high");
            assert_eq!(entry.circuit.output("scl"), Some(true));
            bits.push(entry.circuit.output("sda") == Some(true));
            if bit < 17 {
                entry.circuit.tick().expect("low");
            }
        }
        bits
    }

    fn assert_stop(entry: &mut super::DigitalEntry) {
        entry.circuit.tick().expect("stop low");
        assert_eq!(entry.circuit.output("scl"), Some(false));
        assert_eq!(entry.circuit.output("sda"), Some(false));
        entry.circuit.tick().expect("stop high");
        assert_eq!(entry.circuit.output("scl"), Some(true));
        assert_eq!(entry.circuit.output("sda"), Some(false));
        entry.circuit.tick().expect("stop release");
        assert_eq!(entry.circuit.output("scl"), Some(true));
        assert_eq!(entry.circuit.output("sda"), Some(true));
        entry.circuit.tick().expect("done");
        assert_eq!(entry.circuit.output("done"), Some(true));
    }

    fn expected_bits() -> Vec<bool> {
        vec![
            true, false, true, false, false, false, false, false, false, true, false, true, false,
            false, true, false, true, false,
        ]
    }

    #[test]
    fn i2c_writes_address_and_data() {
        let mut entry = i2c_entry();
        load(&mut entry, 0x50, 0xa5);
        entry.circuit.set_input("ack", true).expect("ack");
        start_transfer(&mut entry);
        let bits = collect_bits(&mut entry);
        assert_eq!(bits, expected_bits());
        assert_eq!(entry.circuit.output("nack"), Some(false));
        assert_stop(&mut entry);
        assert_eq!(entry.circuit.output("busy"), Some(false));
    }

    #[test]
    fn i2c_reports_missing_ack() {
        let mut entry = i2c_entry();
        load(&mut entry, 0x50, 0xa5);
        entry.circuit.set_input("ack", false).expect("ack");
        start_transfer(&mut entry);
        drop(collect_bits(&mut entry));
        assert_eq!(entry.circuit.output("nack"), Some(true));
    }
}
