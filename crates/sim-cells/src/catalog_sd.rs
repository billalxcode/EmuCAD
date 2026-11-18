//! Simplified SPI mode SD card with one 512-byte block.

use crate::catalog_bits::{equals, increment, less_than, mux_tree, register_bits, shift_in};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::catalog_memory::{DATA_BITS, array_cells_n, array_read_n};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 4] = ["reset", "cs", "sclk", "mosi"];
const OUTPUTS: [&str; 2] = ["miso", "busy"];

const IDLE: u32 = 0;
const COMMAND: u32 = 1;
const RESPONSE: u32 = 2;
const READ_TOKEN: u32 = 3;
const READ_DATA: u32 = 4;
const WRITE_TOKEN: u32 = 5;
const WRITE_DATA: u32 = 6;
const WRITE_RESP: u32 = 7;
const BLOCK_BYTES: u32 = 512;
const LAST_BYTE: u32 = 513;

#[derive(Clone, Copy)]
struct SdInputs {
    reset: NetId,
    cs: NetId,
    sclk: NetId,
    mosi: NetId,
}

struct SdState {
    phase: [NetId; 3],
    bit: [NetId; 3],
    cmd_index: [NetId; 3],
    byte: [NetId; 10],
    cmd: [NetId; 8],
    op: [NetId; 8],
    data: [NetId; 8],
    resp: [NetId; 8],
    initialized: NetId,
    resp_phase: NetId,
    cells: Vec<Vec<NetId>>,
}

#[derive(Clone, Copy)]
struct SdPhase {
    reset: NetId,
    flags: [NetId; 8],
    cs_high: NetId,
    active: NetId,
    bit_last: NetId,
}

impl SdPhase {
    fn idle(&self) -> NetId {
        self.flags[IDLE as usize]
    }
    fn command(&self) -> NetId {
        self.flags[COMMAND as usize]
    }
    fn response(&self) -> NetId {
        self.flags[RESPONSE as usize]
    }
    fn read_token(&self) -> NetId {
        self.flags[READ_TOKEN as usize]
    }
    fn read_data(&self) -> NetId {
        self.flags[READ_DATA as usize]
    }
    fn write_token(&self) -> NetId {
        self.flags[WRITE_TOKEN as usize]
    }
    fn write_data(&self) -> NetId {
        self.flags[WRITE_DATA as usize]
    }
    fn write_resp(&self) -> NetId {
        self.flags[WRITE_RESP as usize]
    }
    fn and_last(&self, builder: &mut DigitalBuilder, flag: NetId) -> NetId {
        let bit_active = builder.and(self.bit_last, self.active);
        builder.and(flag, bit_active)
    }
}

#[derive(Clone, Copy)]
struct SdDataEvents {
    token_done: NetId,
    read_done: NetId,
    write_start: NetId,
    write_done: NetId,
    write_resp_done: NetId,
}

#[derive(Clone, Copy)]
struct SdEvents {
    clear: NetId,
    start: NetId,
    command_done: NetId,
    response_done: NetId,
    data: SdDataEvents,
    target: [NetId; 3],
}

struct SdNext {
    phase: [NetId; 3],
    bit: [NetId; 3],
    cmd_index: [NetId; 3],
    byte: [NetId; 10],
    cmd: [NetId; 8],
    op: [NetId; 8],
    data: [NetId; 8],
    resp: [NetId; 8],
    initialized: NetId,
    resp_phase: NetId,
}

#[derive(Clone, Copy)]
struct SdCounters {
    bit: [NetId; 3],
    cmd_index: [NetId; 3],
    byte: [NetId; 10],
}

#[derive(Clone, Copy)]
struct SdRegisters {
    cmd: [NetId; 8],
    op: [NetId; 8],
    data: [NetId; 8],
    resp: [NetId; 8],
    initialized: NetId,
    resp_phase: NetId,
}

impl SdNext {
    fn new(phase: [NetId; 3], counters: SdCounters, registers: SdRegisters) -> Self {
        Self {
            phase,
            bit: counters.bit,
            cmd_index: counters.cmd_index,
            byte: counters.byte,
            cmd: registers.cmd,
            op: registers.op,
            data: registers.data,
            resp: registers.resp,
            initialized: registers.initialized,
            resp_phase: registers.resp_phase,
        }
    }
}

pub(crate) fn sd_card_entry() -> DigitalEntry {
    finish("memory.device.sd_card", |builder| {
        let inputs = sd_inputs(builder);
        let state = sd_state(builder);
        sd_registers(builder, inputs, &state);
        sd_outputs(builder, inputs, &state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn sd_inputs(builder: &mut DigitalBuilder) -> SdInputs {
    SdInputs {
        reset: builder.input("reset"),
        cs: builder.input("cs"),
        sclk: builder.input("sclk"),
        mosi: builder.input("mosi"),
    }
}

fn sd_state(builder: &mut DigitalBuilder) -> SdState {
    SdState {
        phase: [builder.net(), builder.net(), builder.net()],
        bit: [builder.net(), builder.net(), builder.net()],
        cmd_index: [builder.net(), builder.net(), builder.net()],
        byte: std::array::from_fn(|_| builder.net()),
        cmd: std::array::from_fn(|_| builder.net()),
        op: std::array::from_fn(|_| builder.net()),
        data: std::array::from_fn(|_| builder.net()),
        resp: std::array::from_fn(|_| builder.net()),
        initialized: builder.net(),
        resp_phase: builder.net(),
        cells: array_cells_n(builder, BLOCK_BYTES as usize),
    }
}

fn sd_phase(builder: &mut DigitalBuilder, inputs: SdInputs, state: &SdState) -> SdPhase {
    let selected = builder.not(inputs.cs);
    let active = builder.and(selected, inputs.sclk);
    let bit_last = equals(builder, &state.bit, 7);
    let flags = std::array::from_fn(|index| equals(builder, &state.phase, index as u32));
    SdPhase {
        reset: inputs.reset,
        flags,
        cs_high: inputs.cs,
        active,
        bit_last,
    }
}

fn sd_registers(builder: &mut DigitalBuilder, inputs: SdInputs, state: &SdState) {
    let phase = sd_phase(builder, inputs, state);
    let cmd_shift = shift_in(&state.cmd, inputs.mosi);
    let data_shift = shift_in(&state.data, inputs.mosi);
    let events = sd_events(builder, state, &phase, &data_shift);
    let next = sd_next(
        builder,
        inputs,
        state,
        &phase,
        &events,
        &cmd_shift,
        &data_shift,
    );
    sd_memory(builder, inputs, state, &phase, &data_shift);
    sd_store(builder, state, next);
}

fn sd_events(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    data: &[NetId; 8],
) -> SdEvents {
    let (clear, start) = sd_clear_start(builder, phase);
    let command_done = command_done(builder, state, phase);
    let response_done = phase.and_last(builder, phase.response());
    let data = sd_data_events(builder, state, phase, data);
    let target = response_target(builder, &state.op, state.initialized);
    SdEvents {
        clear,
        start,
        command_done,
        response_done,
        data,
        target,
    }
}

fn sd_clear_start(builder: &mut DigitalBuilder, phase: &SdPhase) -> (NetId, NetId) {
    let clear = builder.or(phase.reset, phase.cs_high);
    let start = builder.and(phase.idle(), phase.active);
    (clear, start)
}

fn sd_data_events(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    data: &[NetId; 8],
) -> SdDataEvents {
    let (token_done, read_done) = sd_read_events(builder, state, phase);
    let (write_start, write_done, write_resp_done) = sd_write_events(builder, state, phase, data);
    SdDataEvents {
        token_done,
        read_done,
        write_start,
        write_done,
        write_resp_done,
    }
}

fn sd_read_events(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
) -> (NetId, NetId) {
    let token = phase.and_last(builder, phase.read_token());
    let read = last_phase(builder, state, phase.read_data(), phase);
    (token, read)
}

fn sd_write_events(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    data: &[NetId; 8],
) -> (NetId, NetId, NetId) {
    let token = equals(builder, data, 0xfe);
    let start = phase.and_last(builder, phase.write_token());
    let start = builder.and(start, token);
    let done = last_phase(builder, state, phase.write_data(), phase);
    let resp_active = builder.and(phase.active, state.resp_phase);
    let resp = phase.and_last(builder, phase.write_resp());
    let resp = builder.and(resp, resp_active);
    (start, done, resp)
}

fn last_phase(
    builder: &mut DigitalBuilder,
    state: &SdState,
    flag: NetId,
    phase: &SdPhase,
) -> NetId {
    let last = equals(builder, &state.byte, LAST_BYTE);
    let bit_active = builder.and(phase.bit_last, phase.active);
    let done = builder.and(last, bit_active);
    builder.and(flag, done)
}

fn command_done(builder: &mut DigitalBuilder, state: &SdState, phase: &SdPhase) -> NetId {
    let last_index = equals(builder, &state.cmd_index, 5);
    let done = builder.and(phase.command(), phase.bit_last);
    let active_index = builder.and(phase.active, last_index);
    builder.and(done, active_index)
}

fn response_target(
    builder: &mut DigitalBuilder,
    cmd: &[NetId; 8],
    initialized: NetId,
) -> [NetId; 3] {
    let read = equals(builder, cmd, 0x51);
    let write = equals(builder, cmd, 0x58);
    let read = builder.and(read, initialized);
    let write = builder.and(write, initialized);
    std::array::from_fn(|index| {
        let idle = const_bit(builder, IDLE, index);
        let read_bit = const_bit(builder, READ_TOKEN, index);
        let write_bit = const_bit(builder, WRITE_TOKEN, index);
        let value = builder.mux(idle, read_bit, read);
        builder.mux(value, write_bit, write)
    })
}

fn sd_next(
    builder: &mut DigitalBuilder,
    inputs: SdInputs,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
    cmd_shift: &[NetId; 8],
    data_shift: &[NetId; 8],
) -> SdNext {
    let phase_next = phase_next(builder, state, events);
    let counters = sd_counters(builder, state, phase, events);
    let registers = sd_registers_next(builder, inputs, state, phase, events, cmd_shift, data_shift);
    SdNext::new(phase_next, counters, registers)
}

fn sd_counters(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
) -> SdCounters {
    let bit = bit_next(builder, phase, state.bit);
    let cmd_index = cmd_index_next(builder, state, phase, events);
    let byte = byte_next(builder, state, phase, events);
    SdCounters {
        bit,
        cmd_index,
        byte,
    }
}

fn sd_registers_next(
    builder: &mut DigitalBuilder,
    inputs: SdInputs,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
    cmd_shift: &[NetId; 8],
    data_shift: &[NetId; 8],
) -> SdRegisters {
    let (cmd, op, data) = sd_shift_registers(builder, state, phase, events, cmd_shift, data_shift);
    let mut registers = sd_control_registers(builder, inputs, state, phase, events);
    registers.cmd = cmd;
    registers.op = op;
    registers.data = data;
    registers
}

fn sd_control_registers(
    builder: &mut DigitalBuilder,
    inputs: SdInputs,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
) -> SdRegisters {
    let resp = response_next(builder, state, events);
    let initialized = initialized_next(builder, inputs, state, events);
    let resp_phase = resp_phase_next(builder, state, phase, events);
    SdRegisters {
        cmd: state.cmd,
        op: state.op,
        data: state.data,
        resp,
        initialized,
        resp_phase,
    }
}

fn sd_shift_registers(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
    cmd_shift: &[NetId; 8],
    data_shift: &[NetId; 8],
) -> ([NetId; 8], [NetId; 8], [NetId; 8]) {
    let cmd_load = command_shift(builder, phase);
    let op_load = cmd_latch(builder, state, phase);
    let data_load = data_load(builder, phase);
    let cmd = register_next(builder, &state.cmd, cmd_shift, cmd_load, events.clear);
    let op = register_next(builder, &state.op, cmd_shift, op_load, events.clear);
    let data = register_next(builder, &state.data, data_shift, data_load, events.clear);
    (cmd, op, data)
}

fn register_next(
    builder: &mut DigitalBuilder,
    current: &[NetId; 8],
    value: &[NetId; 8],
    load: NetId,
    clear: NetId,
) -> [NetId; 8] {
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let next = builder.mux(current[index], value[index], load);
        builder.mux(next, zero, clear)
    })
}

fn command_shift(builder: &mut DigitalBuilder, phase: &SdPhase) -> NetId {
    let command = builder.or(phase.idle(), phase.command());
    builder.and(command, phase.active)
}

fn cmd_latch(builder: &mut DigitalBuilder, state: &SdState, phase: &SdPhase) -> NetId {
    let first = equals(builder, &state.cmd_index, 0);
    let command = builder.or(phase.idle(), phase.command());
    let load = builder.and(command, phase.bit_last);
    let active_first = builder.and(phase.active, first);
    builder.and(load, active_first)
}

fn data_load(builder: &mut DigitalBuilder, phase: &SdPhase) -> NetId {
    let write = builder.or(phase.write_token(), phase.write_data());
    builder.and(write, phase.active)
}

fn initialized_next(
    builder: &mut DigitalBuilder,
    inputs: SdInputs,
    state: &SdState,
    events: &SdEvents,
) -> NetId {
    let cmd0 = equals(builder, &state.op, 0x40);
    let cmd1 = equals(builder, &state.op, 0x41);
    let clear = builder.and(events.command_done, cmd0);
    let set = builder.and(events.command_done, cmd1);
    let zero = builder.constant(false);
    let one = builder.constant(true);
    let value = builder.mux(state.initialized, zero, clear);
    let value = builder.mux(value, one, set);
    builder.mux(value, zero, inputs.reset)
}

fn resp_phase_next(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
) -> NetId {
    let last = builder.and(phase.write_resp(), phase.bit_last);
    let last = builder.and(last, phase.active);
    let one = builder.constant(true);
    let zero = builder.constant(false);
    let value = builder.mux(state.resp_phase, one, last);
    let value = builder.mux(value, zero, events.data.write_resp_done);
    builder.mux(value, zero, events.clear)
}

fn response_next(builder: &mut DigitalBuilder, state: &SdState, events: &SdEvents) -> [NetId; 8] {
    let response = command_response(builder, &state.op, state.initialized);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(state.resp[index], response[index], events.command_done);
        builder.mux(value, zero, events.clear)
    })
}

fn command_response(
    builder: &mut DigitalBuilder,
    cmd: &[NetId; 8],
    initialized: NetId,
) -> [NetId; 8] {
    let cmd0 = equals(builder, cmd, 0x40);
    let cmd1 = equals(builder, cmd, 0x41);
    let read = equals(builder, cmd, 0x51);
    let write = equals(builder, cmd, 0x58);
    let data = builder.or(read, write);
    let ready = builder.and(data, initialized);
    let not_initialized = builder.not(initialized);
    let idle_data = builder.and(data, not_initialized);
    let idle = builder.or(cmd0, idle_data);
    let ready = builder.or(cmd1, ready);
    response_byte(builder, idle, ready)
}

fn response_byte(builder: &mut DigitalBuilder, idle: NetId, ready: NetId) -> [NetId; 8] {
    let unknown: [NetId; 8] = std::array::from_fn(|index| const_bit(builder, 0x04, index));
    let idle_bits: [NetId; 8] = std::array::from_fn(|index| const_bit(builder, 0x01, index));
    let ready_bits: [NetId; 8] = std::array::from_fn(|index| const_bit(builder, 0x00, index));
    std::array::from_fn(|index| {
        let value = builder.mux(unknown[index], idle_bits[index], idle);
        builder.mux(value, ready_bits[index], ready)
    })
}

fn phase_next(builder: &mut DigitalBuilder, state: &SdState, events: &SdEvents) -> [NetId; 3] {
    std::array::from_fn(|index| phase_bit(builder, state.phase[index], index, events))
}

fn phase_bit(
    builder: &mut DigitalBuilder,
    current: NetId,
    index: usize,
    events: &SdEvents,
) -> NetId {
    let phase_hold = phase_hold(builder, events);
    let hold = builder.and(current, phase_hold);
    let target = builder.and(events.target[index], events.response_done);
    let mut value = builder.or(hold, target);
    let idle_done = builder.or(events.data.read_done, events.data.write_resp_done);
    let idle = builder.or(idle_done, events.clear);
    value = phase_add(builder, value, COMMAND, index, events.start);
    value = phase_add(builder, value, RESPONSE, index, events.command_done);
    value = phase_add(builder, value, READ_DATA, index, events.data.token_done);
    value = phase_add(builder, value, WRITE_DATA, index, events.data.write_start);
    value = phase_add(builder, value, WRITE_RESP, index, events.data.write_done);
    value = phase_add(builder, value, IDLE, index, idle);
    value
}

fn phase_hold(builder: &mut DigitalBuilder, events: &SdEvents) -> NetId {
    let mut active = builder.or(events.clear, events.start);
    active = builder.or(active, events.command_done);
    active = builder.or(active, events.response_done);
    active = builder.or(active, events.data.token_done);
    active = builder.or(active, events.data.read_done);
    active = builder.or(active, events.data.write_start);
    active = builder.or(active, events.data.write_done);
    active = builder.or(active, events.data.write_resp_done);
    builder.not(active)
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

fn bit_next(builder: &mut DigitalBuilder, phase: &SdPhase, current: [NetId; 3]) -> [NetId; 3] {
    let incremented = increment(builder, &current);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(current[index], incremented[index], phase.active);
        builder.mux(value, zero, phase.bit_last)
    })
}

fn cmd_index_next(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
) -> [NetId; 3] {
    let incremented = increment(builder, &state.cmd_index);
    let last = equals(builder, &state.cmd_index, 5);
    let counting = builder.or(phase.idle(), phase.command());
    let bit_count = builder.and(phase.bit_last, counting);
    let advance = builder.and(phase.active, bit_count);
    let zero = builder.constant(false);
    let wrap = builder.and(advance, last);
    std::array::from_fn(|index| {
        let value = builder.mux(state.cmd_index[index], incremented[index], advance);
        let value = builder.mux(value, zero, wrap);
        let value = builder.mux(value, zero, events.command_done);
        builder.mux(value, zero, events.clear)
    })
}

fn byte_next(
    builder: &mut DigitalBuilder,
    state: &SdState,
    phase: &SdPhase,
    events: &SdEvents,
) -> [NetId; 10] {
    let incremented = increment(builder, &state.byte);
    let zero = builder.constant(false);
    let count = builder.or(phase.read_data(), phase.write_data());
    let bit_active = builder.and(phase.bit_last, phase.active);
    let next = builder.and(count, bit_active);
    std::array::from_fn(|index| {
        let value = builder.mux(state.byte[index], incremented[index], next);
        let value = builder.mux(value, zero, events.data.read_done);
        let value = builder.mux(value, zero, events.data.write_done);
        let value = builder.mux(value, zero, events.data.token_done);
        let value = builder.mux(value, zero, events.data.write_start);
        builder.mux(value, zero, events.clear)
    })
}

fn sd_memory(
    builder: &mut DigitalBuilder,
    inputs: SdInputs,
    state: &SdState,
    phase: &SdPhase,
    data: &[NetId; 8],
) {
    let write = builder.and(phase.write_data(), phase.bit_last);
    let small = less_than(builder, &state.byte, BLOCK_BYTES);
    let active_small = builder.and(phase.active, small);
    let write = builder.and(write, active_small);
    for (index, word) in state.cells.iter().enumerate() {
        let selected = equals(builder, &state.byte[..9], index as u32);
        let word_write = builder.and(write, selected);
        sd_word(builder, inputs.reset, word, data, word_write);
    }
}

fn sd_word(
    builder: &mut DigitalBuilder,
    reset: NetId,
    word: &[NetId],
    data: &[NetId; 8],
    write: NetId,
) {
    let one = builder.constant(true);
    for bit in 0..DATA_BITS {
        let value = builder.mux(word[bit], data[bit], write);
        let value = builder.mux(value, one, reset);
        builder.dff_into(value, word[bit]);
    }
}

fn sd_store(builder: &mut DigitalBuilder, state: &SdState, next: SdNext) {
    let one = builder.constant(true);
    register_bits(builder, &state.phase, &next.phase, one);
    register_bits(builder, &state.bit, &next.bit, one);
    register_bits(builder, &state.cmd_index, &next.cmd_index, one);
    register_bits(builder, &state.byte, &next.byte, one);
    register_bits(builder, &state.cmd, &next.cmd, one);
    register_bits(builder, &state.op, &next.op, one);
    register_bits(builder, &state.data, &next.data, one);
    register_bits(builder, &state.resp, &next.resp, one);
    builder.dff_into(next.initialized, state.initialized);
    builder.dff_into(next.resp_phase, state.resp_phase);
}

fn sd_outputs(builder: &mut DigitalBuilder, inputs: SdInputs, state: &SdState) {
    let phase = sd_phase(builder, inputs, state);
    let resp = byte_mux(builder, &state.resp, &state.bit);
    let token = byte_bit(builder, 0xfe, &state.bit);
    let data = array_read_n(builder, &state.cells, &state.byte[..9]);
    let data_bit = byte_mux(builder, &data, &state.bit);
    let small = less_than(builder, &state.byte, BLOCK_BYTES);
    let ff = byte_bit(builder, 0xff, &state.bit);
    let data_out = builder.mux(ff, data_bit, small);
    let ack = byte_bit(builder, 0x05, &state.bit);
    let write_resp = builder.mux(ack, ff, state.resp_phase);
    let mut miso = byte_bit(builder, 0xff, &state.bit);
    miso = builder.mux(miso, resp, phase.response());
    miso = builder.mux(miso, token, phase.read_token());
    miso = builder.mux(miso, data_out, phase.read_data());
    miso = builder.mux(miso, write_resp, phase.write_resp());
    let busy = builder.not(phase.idle());
    builder.output("miso", miso);
    builder.output("busy", busy);
}

fn byte_mux(builder: &mut DigitalBuilder, bits: &[NetId; 8], bit: &[NetId; 3]) -> NetId {
    let level: Vec<NetId> = (0..8).rev().map(|index| bits[index]).collect();
    mux_tree(builder, level, bit)
}

fn byte_bit(builder: &mut DigitalBuilder, value: u8, bit: &[NetId; 3]) -> NetId {
    let level: Vec<NetId> = (0..8)
        .rev()
        .map(|index| builder.constant(value & (1 << index) != 0))
        .collect();
    mux_tree(builder, level, bit)
}

#[cfg(test)]
mod tests {
    use super::sd_card_entry;

    fn reset(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("reset", true).expect("reset");
        entry.circuit.tick().expect("reset");
        entry.circuit.set_input("reset", false).expect("reset");
        entry.circuit.set_input("cs", true).expect("cs");
        entry.circuit.set_input("sclk", true).expect("sclk");
        entry.circuit.tick().expect("deselect");
        entry.circuit.set_input("cs", false).expect("cs");
    }

    fn send_byte(entry: &mut super::DigitalEntry, value: u8) {
        for index in (0..8).rev() {
            entry
                .circuit
                .set_input("mosi", value & (1 << index) != 0)
                .expect("mosi");
            entry.circuit.settle().expect("settle");
            entry.circuit.tick().expect("bit");
        }
    }

    fn recv_byte(entry: &mut super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..8 {
            entry.circuit.settle().expect("settle");
            if entry.circuit.output("miso") == Some(true) {
                value |= 1 << (7 - index);
            }
            entry.circuit.tick().expect("bit");
        }
        value
    }

    fn command(entry: &mut super::DigitalEntry, cmd: u8) -> u8 {
        send_byte(entry, 0x40 | cmd);
        for _ in 0..4 {
            send_byte(entry, 0);
        }
        send_byte(entry, 0x95);
        recv_byte(entry)
    }

    fn write_block(entry: &mut super::DigitalEntry, pattern: impl Fn(usize) -> u8) -> u8 {
        let response = command(entry, 24);
        assert_eq!(response, 0);
        send_byte(entry, 0xfe);
        for index in 0..512 {
            send_byte(entry, pattern(index));
        }
        send_byte(entry, 0xff);
        send_byte(entry, 0xff);
        let data_response = recv_byte(entry);
        assert_eq!(recv_byte(entry), 0xff);
        data_response
    }

    fn read_block(entry: &mut super::DigitalEntry) -> Vec<u8> {
        let response = command(entry, 17);
        assert_eq!(response, 0);
        assert_eq!(recv_byte(entry), 0xfe);
        let mut data = Vec::with_capacity(512);
        for _ in 0..512 {
            data.push(recv_byte(entry));
        }
        assert_eq!(recv_byte(entry), 0xff);
        assert_eq!(recv_byte(entry), 0xff);
        data
    }

    #[test]
    fn sd_card_writes_and_reads_block() {
        let mut entry = sd_card_entry();
        reset(&mut entry);
        assert_eq!(command(&mut entry, 0), 0x01);
        assert_eq!(command(&mut entry, 1), 0x00);
        let response = write_block(&mut entry, |index| index as u8);
        assert_eq!(response, 0x05);
        let data = read_block(&mut entry);
        let expected: Vec<u8> = (0..512).map(|index| index as u8).collect();
        assert_eq!(data, expected);
    }

    #[test]
    fn sd_card_rejects_uninitialized_read() {
        let mut entry = sd_card_entry();
        reset(&mut entry);
        assert_eq!(command(&mut entry, 17), 0x01);
    }

    #[test]
    fn sd_card_rejects_unknown_command() {
        let mut entry = sd_card_entry();
        reset(&mut entry);
        assert_eq!(command(&mut entry, 0x3f), 0x04);
    }
}
