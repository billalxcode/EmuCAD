//! Digital SPI flash memory with byte program and chip erase.

use crate::catalog_bits::{equals, increment, mux_tree, register_bits, shift_in};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::catalog_memory::{DATA_BITS, array_cells, array_read};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 4] = ["reset", "cs", "sclk", "mosi"];
const OUTPUTS: [&str; 3] = ["miso", "wel", "busy"];

const IDLE: u32 = 0;
const COMMAND: u32 = 1;
const ADDRESS: u32 = 2;
const READ: u32 = 3;
const PROGRAM: u32 = 4;
const STATUS: u32 = 5;
const DONE: u32 = 6;

#[derive(Clone, Copy)]
struct FlashInputs {
    reset: NetId,
    cs: NetId,
    sclk: NetId,
    mosi: NetId,
}

struct FlashState {
    phase: [NetId; 3],
    count: [NetId; 4],
    command: [NetId; 8],
    address: [NetId; 7],
    data: [NetId; 8],
    wel: NetId,
    cells: Vec<Vec<NetId>>,
}

#[derive(Clone, Copy)]
struct FlashPhase {
    idle: NetId,
    command: NetId,
    address: NetId,
    read: NetId,
    program: NetId,
    status: NetId,
    active: NetId,
    phase_done: NetId,
}

#[derive(Clone, Copy)]
struct PhaseConditions {
    start: NetId,
    command_done: NetId,
    address_done: NetId,
    status_done: NetId,
}

#[derive(Clone, Copy)]
struct PhaseSignals {
    start: NetId,
    command_done: NetId,
    address_done: NetId,
    status_done: NetId,
    clear: NetId,
    hold: NetId,
    target: [NetId; 3],
    address_target: [NetId; 3],
}

#[derive(Clone, Copy)]
struct FlashControl {
    phase: [NetId; 3],
    count: [NetId; 4],
    wel: NetId,
    erase: NetId,
    program: NetId,
}

struct FlashNext {
    phase: [NetId; 3],
    count: [NetId; 4],
    command: [NetId; 8],
    address: [NetId; 7],
    data: [NetId; 8],
    wel: NetId,
}

pub(crate) fn spi_flash_entry() -> DigitalEntry {
    finish("memory.device.spi_flash", |builder| {
        let inputs = flash_inputs(builder);
        let state = flash_state(builder);
        flash_registers(builder, inputs, &state);
        flash_outputs(builder, inputs, &state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn flash_inputs(builder: &mut DigitalBuilder) -> FlashInputs {
    FlashInputs {
        reset: builder.input("reset"),
        cs: builder.input("cs"),
        sclk: builder.input("sclk"),
        mosi: builder.input("mosi"),
    }
}

fn flash_state(builder: &mut DigitalBuilder) -> FlashState {
    FlashState {
        phase: [builder.net(), builder.net(), builder.net()],
        count: std::array::from_fn(|_| builder.net()),
        command: std::array::from_fn(|_| builder.net()),
        address: std::array::from_fn(|_| builder.net()),
        data: std::array::from_fn(|_| builder.net()),
        wel: builder.net(),
        cells: array_cells(builder),
    }
}

fn flash_phase(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
) -> FlashPhase {
    let selected = builder.not(inputs.cs);
    let active = builder.and(selected, inputs.sclk);
    let last = equals(builder, &state.count, 7);
    let phase_done = builder.and(active, last);
    FlashPhase {
        idle: equals(builder, &state.phase, IDLE),
        command: equals(builder, &state.phase, COMMAND),
        address: equals(builder, &state.phase, ADDRESS),
        read: equals(builder, &state.phase, READ),
        program: equals(builder, &state.phase, PROGRAM),
        status: equals(builder, &state.phase, STATUS),
        active,
        phase_done,
    }
}

fn flash_registers(builder: &mut DigitalBuilder, inputs: FlashInputs, state: &FlashState) {
    let phase = flash_phase(builder, inputs, state);
    let clear = builder.or(inputs.reset, inputs.cs);
    let (command, address, data) = shift_registers(builder, inputs, state, &phase, clear);
    let signals = phase_signals(builder, inputs, &phase, &command);
    let control = flash_control(builder, inputs, state, &phase, &signals, &command);
    flash_finish(builder, state, control, command, address, data);
}

fn flash_finish(
    builder: &mut DigitalBuilder,
    state: &FlashState,
    control: FlashControl,
    command: [NetId; 8],
    address: [NetId; 7],
    data: [NetId; 8],
) {
    flash_memory(
        builder,
        state,
        &control.erase,
        &control.program,
        &state.address,
        &data,
    );
    let next = flash_next_value(control, command, address, data);
    flash_store(builder, state, next);
}

fn flash_next_value(
    control: FlashControl,
    command: [NetId; 8],
    address: [NetId; 7],
    data: [NetId; 8],
) -> FlashNext {
    FlashNext {
        phase: control.phase,
        count: control.count,
        command,
        address,
        data,
        wel: control.wel,
    }
}

fn flash_control(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
    phase: &FlashPhase,
    signals: &PhaseSignals,
    command: &[NetId; 8],
) -> FlashControl {
    let phase_next = phase_next(builder, state, signals);
    let count = count_next(builder, state, phase, signals.clear);
    let erase = erase_value(builder, inputs, signals, command);
    let (wel, program) = wel_and_program(builder, inputs, state, phase, signals, command);
    FlashControl {
        phase: phase_next,
        count,
        wel,
        erase,
        program,
    }
}

fn wel_and_program(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
    phase: &FlashPhase,
    signals: &PhaseSignals,
    command: &[NetId; 8],
) -> (NetId, NetId) {
    let wel = wel_value(builder, inputs, state, phase, signals, command);
    let program = program_value(builder, phase, state.wel);
    (wel, program)
}

fn program_value(builder: &mut DigitalBuilder, phase: &FlashPhase, wel: NetId) -> NetId {
    let ready = builder.and(phase.program, wel);
    builder.and(phase.phase_done, ready)
}

fn shift_registers(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
    phase: &FlashPhase,
    clear: NetId,
) -> ([NetId; 8], [NetId; 7], [NetId; 8]) {
    let command_phase = builder.or(phase.idle, phase.command);
    let command_load = builder.and(phase.active, command_phase);
    let data_load = builder.and(phase.active, phase.program);
    let command = shift_next(builder, &state.command, inputs.mosi, command_load, clear);
    let data = shift_next(builder, &state.data, inputs.mosi, data_load, clear);
    let address = address_next(builder, inputs, state, phase, clear);
    (command, address, data)
}

fn shift_next<const N: usize>(
    builder: &mut DigitalBuilder,
    current: &[NetId; N],
    input: NetId,
    load: NetId,
    clear: NetId,
) -> [NetId; N] {
    let shifted = shift_in(current, input);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(current[index], shifted[index], load);
        builder.mux(value, zero, clear)
    })
}

fn address_next(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
    phase: &FlashPhase,
    clear: NetId,
) -> [NetId; 7] {
    let shifted = shift_in(&state.address, inputs.mosi);
    let incremented = increment(builder, &state.address);
    let load = builder.and(phase.active, phase.address);
    let data_phase = builder.or(phase.read, phase.program);
    let advance = builder.and(phase.phase_done, data_phase);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(state.address[index], shifted[index], load);
        let value = builder.mux(value, incremented[index], advance);
        builder.mux(value, zero, clear)
    })
}

fn phase_signals(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    phase: &FlashPhase,
    command: &[NetId; 8],
) -> PhaseSignals {
    let conditions = phase_conditions(builder, phase);
    let clear = builder.or(inputs.reset, inputs.cs);
    let hold = phase_hold(builder, clear, &conditions);
    PhaseSignals {
        start: conditions.start,
        command_done: conditions.command_done,
        address_done: conditions.address_done,
        status_done: conditions.status_done,
        clear,
        hold,
        target: command_target(builder, command),
        address_target: address_target(builder, command),
    }
}

fn phase_conditions(builder: &mut DigitalBuilder, phase: &FlashPhase) -> PhaseConditions {
    let start = builder.and(phase.active, phase.idle);
    let command = builder.and(phase.command, phase.phase_done);
    let address = builder.and(phase.address, phase.phase_done);
    let status = builder.and(phase.status, phase.phase_done);
    PhaseConditions {
        start,
        command_done: command,
        address_done: address,
        status_done: status,
    }
}

fn phase_hold(builder: &mut DigitalBuilder, clear: NetId, conditions: &PhaseConditions) -> NetId {
    let address_or_status = builder.or(conditions.address_done, conditions.status_done);
    let done = builder.or(conditions.command_done, address_or_status);
    let active_done = builder.or(conditions.start, done);
    let clear_or_done = builder.or(clear, active_done);
    builder.not(clear_or_done)
}

fn command_target(builder: &mut DigitalBuilder, command: &[NetId; 8]) -> [NetId; 3] {
    let status = equals(builder, command, 0x05);
    let read = equals(builder, command, 0x03);
    let program = equals(builder, command, 0x02);
    let address = builder.or(read, program);
    phase_bits(builder, status, address)
}

fn address_target(builder: &mut DigitalBuilder, command: &[NetId; 8]) -> [NetId; 3] {
    let read = equals(builder, command, 0x03);
    std::array::from_fn(|index| {
        let program = const_bit(builder, PROGRAM, index);
        let value = const_bit(builder, READ, index);
        builder.mux(program, value, read)
    })
}

fn phase_bits(builder: &mut DigitalBuilder, status: NetId, address: NetId) -> [NetId; 3] {
    std::array::from_fn(|index| {
        let done = const_bit(builder, DONE, index);
        let status_bit = const_bit(builder, STATUS, index);
        let address_bit = const_bit(builder, ADDRESS, index);
        let value = builder.mux(done, status_bit, status);
        builder.mux(value, address_bit, address)
    })
}

fn const_bit(builder: &mut DigitalBuilder, value: u32, index: usize) -> NetId {
    builder.constant(value & (1 << index) != 0)
}

fn phase_next(
    builder: &mut DigitalBuilder,
    state: &FlashState,
    signals: &PhaseSignals,
) -> [NetId; 3] {
    std::array::from_fn(|index| phase_bit(builder, state.phase[index], index, signals))
}

fn phase_bit(
    builder: &mut DigitalBuilder,
    phase: NetId,
    index: usize,
    signals: &PhaseSignals,
) -> NetId {
    let hold = builder.and(phase, signals.hold);
    let start = phase_const(builder, COMMAND, index, signals.start);
    let command = builder.and(signals.target[index], signals.command_done);
    let address = builder.and(signals.address_target[index], signals.address_done);
    let status = phase_const(builder, STATUS, index, signals.status_done);
    let clear = phase_const(builder, IDLE, index, signals.clear);
    let value = builder.or(hold, start);
    let value = builder.or(value, command);
    let value = builder.or(value, address);
    let value = builder.or(value, status);
    builder.or(value, clear)
}

fn phase_const(builder: &mut DigitalBuilder, code: u32, index: usize, select: NetId) -> NetId {
    let bit = const_bit(builder, code, index);
    builder.and(bit, select)
}

fn count_next(
    builder: &mut DigitalBuilder,
    state: &FlashState,
    phase: &FlashPhase,
    clear: NetId,
) -> [NetId; 4] {
    let incremented = increment(builder, &state.count);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(state.count[index], incremented[index], phase.active);
        let value = builder.mux(value, zero, phase.phase_done);
        builder.mux(value, zero, clear)
    })
}

fn erase_value(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    signals: &PhaseSignals,
    command: &[NetId; 8],
) -> NetId {
    let erase = equals(builder, command, 0xc7);
    let erase = builder.and(signals.command_done, erase);
    builder.or(inputs.reset, erase)
}

fn wel_value(
    builder: &mut DigitalBuilder,
    inputs: FlashInputs,
    state: &FlashState,
    phase: &FlashPhase,
    signals: &PhaseSignals,
    command: &[NetId; 8],
) -> NetId {
    let wren = equals(builder, command, 0x06);
    let set = builder.and(signals.command_done, wren);
    let erase = equals(builder, command, 0xc7);
    let erase = builder.and(signals.command_done, erase);
    let program = builder.and(phase.phase_done, phase.program);
    let clear_program = builder.or(erase, program);
    let clear = builder.or(inputs.reset, clear_program);
    let one = builder.constant(true);
    let zero = builder.constant(false);
    let value = builder.mux(state.wel, one, set);
    builder.mux(value, zero, clear)
}

fn flash_memory(
    builder: &mut DigitalBuilder,
    state: &FlashState,
    erase: &NetId,
    program: &NetId,
    address: &[NetId; 7],
    data: &[NetId; 8],
) {
    for (index, word) in state.cells.iter().enumerate() {
        let selected = equals(builder, address, index as u32);
        let word_program = builder.and(*program, selected);
        flash_word(builder, word, data, word_program, *erase);
    }
}

fn flash_word(
    builder: &mut DigitalBuilder,
    word: &[NetId],
    data: &[NetId; 8],
    program: NetId,
    erase: NetId,
) {
    let one = builder.constant(true);
    for bit in 0..DATA_BITS {
        let masked = builder.and(word[bit], data[bit]);
        let value = builder.mux(word[bit], masked, program);
        let value = builder.mux(value, one, erase);
        builder.dff_into(value, word[bit]);
    }
}

fn flash_store(builder: &mut DigitalBuilder, state: &FlashState, next: FlashNext) {
    let one = builder.constant(true);
    register_bits(builder, &state.phase, &next.phase, one);
    register_bits(builder, &state.count, &next.count, one);
    register_bits(builder, &state.command, &next.command, one);
    register_bits(builder, &state.address, &next.address, one);
    register_bits(builder, &state.data, &next.data, one);
    builder.dff_into(next.wel, state.wel);
}

fn flash_outputs(builder: &mut DigitalBuilder, inputs: FlashInputs, state: &FlashState) {
    let phase = flash_phase(builder, inputs, state);
    let data = array_read(builder, &state.cells, &state.address);
    let mut level: Vec<NetId> = (0..DATA_BITS).rev().map(|index| data[index]).collect();
    level.truncate(8);
    let read = mux_tree(builder, level, &state.count[..3]);
    let first = equals(builder, &state.count, 0);
    let status = builder.and(state.wel, first);
    let zero = builder.constant(false);
    let miso = builder.mux(zero, read, phase.read);
    let miso = builder.mux(miso, status, phase.status);
    builder.output("miso", miso);
    builder.output("wel", state.wel);
    builder.output("busy", phase.active);
}

#[cfg(test)]
mod tests {
    use super::spi_flash_entry;

    fn reset(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("reset", true).expect("reset");
        entry.circuit.tick().expect("reset");
        entry.circuit.set_input("reset", false).expect("reset");
        entry.circuit.set_input("cs", true).expect("cs");
        entry.circuit.tick().expect("deselect");
        entry.circuit.set_input("cs", false).expect("cs");
    }

    fn send(entry: &mut super::DigitalEntry, value: u8) {
        for index in (0..8).rev() {
            entry
                .circuit
                .set_input("mosi", value & (1 << index) != 0)
                .expect("mosi");
            entry.circuit.set_input("sclk", true).expect("sclk");
            entry.circuit.tick().expect("bit");
        }
    }

    fn receive(entry: &mut super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..8 {
            if entry.circuit.output("miso") == Some(true) {
                value |= 1 << (7 - index);
            }
            if index < 7 {
                entry.circuit.tick().expect("read bit");
            }
        }
        value
    }

    fn deselect(entry: &mut super::DigitalEntry) {
        entry.circuit.set_input("cs", true).expect("cs");
        entry.circuit.tick().expect("deselect");
        entry.circuit.set_input("cs", false).expect("cs");
    }

    #[test]
    fn spi_flash_reset_is_blank() {
        let mut entry = spi_flash_entry();
        reset(&mut entry);
        send(&mut entry, 0x03);
        send(&mut entry, 0x00);
        assert_eq!(receive(&mut entry), 0xff);
    }

    #[test]
    fn spi_flash_programs_and_reads() {
        let mut entry = spi_flash_entry();
        reset(&mut entry);
        send(&mut entry, 0x06);
        assert_eq!(entry.circuit.output("wel"), Some(true));
        deselect(&mut entry);
        send(&mut entry, 0x02);
        send(&mut entry, 0x00);
        send(&mut entry, 0x0f);
        assert_eq!(entry.circuit.output("wel"), Some(false));
        deselect(&mut entry);
        send(&mut entry, 0x03);
        send(&mut entry, 0x00);
        assert_eq!(receive(&mut entry), 0x0f);
    }

    #[test]
    fn spi_flash_erases_to_ff() {
        let mut entry = spi_flash_entry();
        reset(&mut entry);
        send(&mut entry, 0x06);
        deselect(&mut entry);
        send(&mut entry, 0x02);
        send(&mut entry, 0x01);
        send(&mut entry, 0x00);
        deselect(&mut entry);
        send(&mut entry, 0x06);
        deselect(&mut entry);
        send(&mut entry, 0xc7);
        deselect(&mut entry);
        send(&mut entry, 0x03);
        send(&mut entry, 0x01);
        assert_eq!(receive(&mut entry), 0xff);
    }
}
