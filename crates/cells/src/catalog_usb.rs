//! USB full-speed token packet transmitter.

use crate::catalog_bits::{equals, increment, mux_tree};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 17] = [
    "reset", "start", "pid0", "pid1", "pid2", "pid3", "addr0", "addr1", "addr2", "addr3", "addr4",
    "addr5", "addr6", "ep0", "ep1", "ep2", "ep3",
];
const OUTPUTS: [&str; 4] = ["dp", "dm", "busy", "done"];
const PID_NAMES: [&str; 4] = ["pid0", "pid1", "pid2", "pid3"];
const ADDRESS_NAMES: [&str; 7] = [
    "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6",
];
const ENDPOINT_NAMES: [&str; 4] = ["ep0", "ep1", "ep2", "ep3"];

const IDLE: u32 = 0;
const DATA: u32 = 1;
const EOP: u32 = 2;
const DONE: u32 = 3;

#[derive(Clone, Copy)]
struct UsbInputs {
    reset: NetId,
    start: NetId,
    pid: [NetId; 4],
    address: [NetId; 7],
    endpoint: [NetId; 4],
}

struct UsbState {
    phase: [NetId; 2],
    index: [NetId; 6],
    run: [NetId; 3],
    line: NetId,
    stuff: NetId,
    eop: [NetId; 2],
    dp: NetId,
    dm: NetId,
}

#[derive(Clone, Copy)]
struct UsbPhase {
    reset: NetId,
    data: NetId,
    eop: NetId,
    done: NetId,
    start_load: NetId,
}

#[derive(Clone, Copy)]
struct UsbNext {
    phase: [NetId; 2],
    index: [NetId; 6],
    run: [NetId; 3],
    line: NetId,
    stuff: NetId,
    eop: [NetId; 2],
    dp: NetId,
    dm: NetId,
}

#[derive(Clone, Copy)]
struct UsbCounters {
    index: [NetId; 6],
    run: [NetId; 3],
    eop: [NetId; 2],
}

#[derive(Clone, Copy)]
struct UsbLines {
    line: NetId,
    stuff: NetId,
    dp: NetId,
    dm: NetId,
}

impl UsbNext {
    fn new(phase: [NetId; 2], counters: UsbCounters, lines: UsbLines) -> Self {
        Self {
            phase,
            index: counters.index,
            run: counters.run,
            line: lines.line,
            stuff: lines.stuff,
            eop: counters.eop,
            dp: lines.dp,
            dm: lines.dm,
        }
    }
}

pub(crate) fn usb_token_entry() -> DigitalEntry {
    finish("mcu.serial.usb", |builder| {
        let inputs = usb_inputs(builder);
        let state = usb_state(builder);
        usb_registers(builder, inputs, &state);
        usb_outputs(builder, inputs, &state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn usb_inputs(builder: &mut DigitalBuilder) -> UsbInputs {
    UsbInputs {
        reset: builder.input("reset"),
        start: builder.input("start"),
        pid: std::array::from_fn(|index| builder.input(PID_NAMES[index])),
        address: std::array::from_fn(|index| builder.input(ADDRESS_NAMES[index])),
        endpoint: std::array::from_fn(|index| builder.input(ENDPOINT_NAMES[index])),
    }
}

fn usb_state(builder: &mut DigitalBuilder) -> UsbState {
    UsbState {
        phase: [builder.net(), builder.net()],
        index: std::array::from_fn(|_| builder.net()),
        run: [builder.net(), builder.net(), builder.net()],
        line: builder.net(),
        stuff: builder.net(),
        eop: [builder.net(), builder.net()],
        dp: builder.net(),
        dm: builder.net(),
    }
}

fn usb_phase(builder: &mut DigitalBuilder, inputs: UsbInputs, state: &UsbState) -> UsbPhase {
    let idle = equals(builder, &state.phase, IDLE);
    let done = equals(builder, &state.phase, DONE);
    let not_busy = builder.or(idle, done);
    let start_load = builder.and(inputs.start, not_busy);
    UsbPhase {
        reset: inputs.reset,
        data: equals(builder, &state.phase, DATA),
        eop: equals(builder, &state.phase, EOP),
        done,
        start_load,
    }
}

fn usb_registers(builder: &mut DigitalBuilder, inputs: UsbInputs, state: &UsbState) {
    let phase = usb_phase(builder, inputs, state);
    let raw = token_bit(builder, inputs, state);
    let schedule = schedule_of(builder, &phase, state, raw);
    let next = usb_next(builder, state, &phase, raw, schedule);
    usb_store(builder, state, next);
}

fn usb_next(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    raw: NetId,
    schedule: NetId,
) -> UsbNext {
    let phase_next = phase_next(builder, state, phase, schedule);
    let counters = usb_counters(builder, state, phase, raw);
    let lines = usb_lines(builder, state, phase, raw, schedule);
    UsbNext::new(phase_next, counters, lines)
}

fn usb_counters(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    raw: NetId,
) -> UsbCounters {
    let index = index_next(builder, phase, state);
    let run = run_next(builder, phase, state, raw);
    let eop = eop_next(builder, phase, state);
    UsbCounters { index, run, eop }
}

fn usb_lines(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    raw: NetId,
    schedule: NetId,
) -> UsbLines {
    let line = line_next(builder, phase, state, raw);
    let stuff = stuff_next(builder, phase, schedule);
    let (dp, dm) = line_outputs(builder, phase, state, line);
    UsbLines {
        line,
        stuff,
        dp,
        dm,
    }
}

fn schedule_of(
    builder: &mut DigitalBuilder,
    phase: &UsbPhase,
    state: &UsbState,
    raw: NetId,
) -> NetId {
    let run_full = equals(builder, &state.run, 5);
    let stuffable = builder.and(phase.data, raw);
    let not_stuff = builder.not(state.stuff);
    let stuffable = builder.and(stuffable, not_stuff);
    builder.and(stuffable, run_full)
}

fn phase_next(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    schedule: NetId,
) -> [NetId; 2] {
    std::array::from_fn(|bit| phase_bit(builder, state, phase, schedule, bit))
}

fn phase_bit(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    schedule: NetId,
    bit: usize,
) -> NetId {
    let end = phase_end(builder, state, phase, schedule);
    let eop_last = equals(builder, &state.eop, 2);
    let (data, eop, done) = phase_constants(builder, bit);
    let data_next = builder.mux(data, eop, end);
    let eop_next = builder.mux(eop, done, eop_last);
    let mut value = state.phase[bit];
    value = builder.mux(value, data, phase.start_load);
    value = builder.mux(value, data_next, phase.data);
    value = builder.mux(value, eop_next, phase.eop);
    let idle = const_bit(builder, IDLE, bit);
    builder.mux(value, idle, phase.reset)
}

fn phase_constants(builder: &mut DigitalBuilder, bit: usize) -> (NetId, NetId, NetId) {
    let data = const_bit(builder, DATA, bit);
    let eop = const_bit(builder, EOP, bit);
    let done = const_bit(builder, DONE, bit);
    (data, eop, done)
}

fn phase_end(
    builder: &mut DigitalBuilder,
    state: &UsbState,
    phase: &UsbPhase,
    schedule: NetId,
) -> NetId {
    let index_last = equals(builder, &state.index, 31);
    let index_end = equals(builder, &state.index, 32);
    let data_done = builder.and(phase.data, schedule);
    let data_done = builder.not(data_done);
    let last = builder.and(index_last, data_done);
    let stuff_end = builder.and(state.stuff, index_end);
    builder.or(last, stuff_end)
}

fn index_next(builder: &mut DigitalBuilder, phase: &UsbPhase, state: &UsbState) -> [NetId; 6] {
    let incremented = increment(builder, &state.index);
    let one = [
        builder.constant(true),
        builder.constant(false),
        builder.constant(false),
        builder.constant(false),
        builder.constant(false),
        builder.constant(false),
    ];
    let not_stuff = builder.not(state.stuff);
    let advance = builder.and(phase.data, not_stuff);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let value = builder.mux(state.index[bit], incremented[bit], advance);
        let value = builder.mux(value, one[bit], phase.start_load);
        builder.mux(value, zero, phase.reset)
    })
}

fn run_next(
    builder: &mut DigitalBuilder,
    phase: &UsbPhase,
    state: &UsbState,
    raw: NetId,
) -> [NetId; 3] {
    let incremented = increment(builder, &state.run);
    let not_stuff = builder.not(state.stuff);
    let normal = builder.and(phase.data, not_stuff);
    let active = builder.and(normal, raw);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let value = builder.mux(zero, incremented[bit], active);
        let value = builder.mux(value, zero, state.stuff);
        let value = builder.mux(value, zero, phase.start_load);
        builder.mux(value, zero, phase.reset)
    })
}

fn line_next(
    builder: &mut DigitalBuilder,
    phase: &UsbPhase,
    state: &UsbState,
    raw: NetId,
) -> NetId {
    let not_line = builder.not(state.line);
    let normal = builder.mux(not_line, state.line, raw);
    let data = builder.mux(normal, not_line, state.stuff);
    let one = builder.constant(true);
    let zero = builder.constant(false);
    let value = builder.mux(state.line, data, phase.data);
    let value = builder.mux(value, one, phase.eop);
    let value = builder.mux(value, one, phase.done);
    let value = builder.mux(value, one, phase.reset);
    builder.mux(value, zero, phase.start_load)
}

fn stuff_next(builder: &mut DigitalBuilder, phase: &UsbPhase, schedule: NetId) -> NetId {
    let zero = builder.constant(false);
    let value = builder.mux(zero, schedule, phase.data);
    builder.mux(value, zero, phase.reset)
}

fn eop_next(builder: &mut DigitalBuilder, phase: &UsbPhase, state: &UsbState) -> [NetId; 2] {
    let incremented = increment(builder, &state.eop);
    let zero = builder.constant(false);
    std::array::from_fn(|bit| {
        let value = builder.mux(state.eop[bit], incremented[bit], phase.eop);
        let value = builder.mux(value, zero, phase.start_load);
        builder.mux(value, zero, phase.reset)
    })
}

fn usb_store(builder: &mut DigitalBuilder, state: &UsbState, next: UsbNext) {
    let one = builder.constant(true);
    crate::catalog_bits::register_bits(builder, &state.phase, &next.phase, one);
    crate::catalog_bits::register_bits(builder, &state.index, &next.index, one);
    crate::catalog_bits::register_bits(builder, &state.run, &next.run, one);
    crate::catalog_bits::register_bits(builder, &state.eop, &next.eop, one);
    builder.dff_into(next.line, state.line);
    builder.dff_into(next.stuff, state.stuff);
    builder.dff_into(next.dp, state.dp);
    builder.dff_into(next.dm, state.dm);
}

fn usb_outputs(builder: &mut DigitalBuilder, inputs: UsbInputs, state: &UsbState) {
    let phase = usb_phase(builder, inputs, state);
    let busy = builder.or(phase.data, phase.eop);
    builder.output("dp", state.dp);
    builder.output("dm", state.dm);
    builder.output("busy", busy);
    builder.output("done", phase.done);
}

fn line_outputs(
    builder: &mut DigitalBuilder,
    phase: &UsbPhase,
    state: &UsbState,
    line: NetId,
) -> (NetId, NetId) {
    let one = builder.constant(true);
    let zero = builder.constant(false);
    let data = builder.or(phase.data, phase.start_load);
    let not_line = builder.not(line);
    let eop_last = equals(builder, &state.eop, 2);
    let eop_dp = eop_last;
    let mut dp = one;
    let mut dm = zero;
    dp = builder.mux(dp, line, data);
    dm = builder.mux(dm, not_line, data);
    dp = builder.mux(dp, eop_dp, phase.eop);
    dm = builder.mux(dm, zero, phase.eop);
    (dp, dm)
}

fn token_bit(builder: &mut DigitalBuilder, inputs: UsbInputs, state: &UsbState) -> NetId {
    let mut level = header_level(builder, inputs);
    level.extend(field_level(builder, inputs));
    while level.len() < 64 {
        level.push(builder.constant(false));
    }
    mux_tree(builder, level, &state.index)
}

fn header_level(builder: &mut DigitalBuilder, inputs: UsbInputs) -> Vec<NetId> {
    let mut level = Vec::with_capacity(16);
    for index in 0..8 {
        level.push(builder.constant(index == 7));
    }
    for bit in inputs.pid {
        level.push(bit);
    }
    for bit in inputs.pid {
        level.push(builder.not(bit));
    }
    level
}

fn field_level(builder: &mut DigitalBuilder, inputs: UsbInputs) -> Vec<NetId> {
    let mut level = Vec::with_capacity(16);
    level.extend(inputs.address);
    level.extend(inputs.endpoint);
    level.extend(crc5(builder, inputs));
    level
}

fn crc5(builder: &mut DigitalBuilder, inputs: UsbInputs) -> [NetId; 5] {
    let one = builder.constant(true);
    let mut crc = [one; 5];
    let field: [NetId; 11] = std::array::from_fn(|index| {
        if index < 7 {
            inputs.address[index]
        } else {
            inputs.endpoint[index - 7]
        }
    });
    for bit in field {
        let feedback = builder.xor(crc[0], bit);
        crc = shift_right(builder, crc);
        crc[2] = builder.xor(crc[2], feedback);
        crc[4] = builder.xor(crc[4], feedback);
    }
    std::array::from_fn(|index| builder.xor(crc[index], one))
}

fn shift_right(builder: &mut DigitalBuilder, crc: [NetId; 5]) -> [NetId; 5] {
    let zero = builder.constant(false);
    [crc[1], crc[2], crc[3], crc[4], zero]
}

fn const_bit(builder: &mut DigitalBuilder, value: u32, index: usize) -> NetId {
    builder.constant(value & (1 << index) != 0)
}

#[cfg(test)]
mod tests {
    use super::usb_token_entry;

    fn load(entry: &mut super::DigitalEntry, pid: u8, address: u8, endpoint: u8) {
        for bit in 0..4 {
            entry
                .circuit
                .set_input(&format!("pid{bit}"), pid & (1 << bit) != 0)
                .expect("pid");
            entry
                .circuit
                .set_input(&format!("ep{bit}"), endpoint & (1 << bit) != 0)
                .expect("ep");
        }
        for bit in 0..7 {
            entry
                .circuit
                .set_input(&format!("addr{bit}"), address & (1 << bit) != 0)
                .expect("addr");
        }
        entry.circuit.set_input("start", false).expect("start");
    }

    fn collect(entry: &mut super::DigitalEntry) -> (Vec<bool>, Vec<bool>) {
        entry.circuit.set_input("start", true).expect("start");
        entry.circuit.settle().expect("settle");
        entry.circuit.tick().expect("start");
        entry.circuit.set_input("start", false).expect("start");
        let mut dp = Vec::new();
        let mut dm = Vec::new();
        for _ in 0..128 {
            dp.push(entry.circuit.output("dp") == Some(true));
            dm.push(entry.circuit.output("dm") == Some(true));
            if entry.circuit.output("done") == Some(true) {
                return (dp, dm);
            }
            entry.circuit.tick().expect("bit");
        }
        panic!("usb packet did not finish")
    }

    fn bits(value: &str) -> Vec<bool> {
        value.chars().map(|bit| bit == '1').collect()
    }

    #[test]
    fn usb_sof_token_matches_reference() {
        let mut entry = usb_token_entry();
        load(&mut entry, 0x5, 0x10, 0xe);
        let (dp, dm) = collect(&mut entry);
        assert_eq!(dp, bits("01010100011011001010010111110010001"));
        assert_eq!(dm, bits("10101011100100110101101000001101000"));
    }

    #[test]
    fn usb_in_token_stuffs_six_ones() {
        let mut entry = usb_token_entry();
        load(&mut entry, 0x9, 0x7f, 0xf);
        let (dp, dm) = collect(&mut entry);
        assert_eq!(dp, bits("010101000100111000000011111101001001"));
        assert_eq!(dm, bits("101010111011000111111100000010110000"));
    }
}
