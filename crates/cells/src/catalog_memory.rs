//! Gate-level SRAM cell and digital memory arrays.

use crate::catalog_bits::{equals, increment, mux_tree, register_bits};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

pub(crate) const ADDRESS_BITS: usize = 7;
pub(crate) const DATA_BITS: usize = 8;
pub(crate) const WORDS: usize = 1 << ADDRESS_BITS;
const ADDRESS: [&str; ADDRESS_BITS] = [
    "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6",
];
const DATA_IN: [&str; DATA_BITS] = [
    "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7",
];
const DATA_OUT: [&str; DATA_BITS] = [
    "dout0", "dout1", "dout2", "dout3", "dout4", "dout5", "dout6", "dout7",
];

pub(crate) fn memory_array_entry(id: &'static str, retain: bool) -> DigitalEntry {
    finish(id, |builder| {
        let reset = builder.input("reset");
        let write_enable = builder.input("we");
        let address = address_inputs(builder);
        let data_in = data_inputs(builder);
        let cells = array_cells(builder);
        sram_write(
            builder,
            &cells,
            reset,
            write_enable,
            retain,
            &address,
            &data_in,
        );
        let data_out = array_read(builder, &cells, &address);
        for (index, net) in data_out.iter().enumerate() {
            builder.output(DATA_OUT[index], *net);
        }
        (sram_inputs(), DATA_OUT.to_vec())
    })
}

const ROM_INPUTS: [&str; 16] = [
    "load", "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6", "din0", "din1", "din2",
    "din3", "din4", "din5", "din6", "din7",
];

pub(crate) fn rom_array_entry(id: &'static str) -> DigitalEntry {
    rom_entry(id, false)
}

pub(crate) fn prom_array_entry(id: &'static str) -> DigitalEntry {
    rom_entry(id, true)
}

fn rom_entry(id: &'static str, one_time: bool) -> DigitalEntry {
    finish(id, |builder| {
        let load = builder.input("load");
        let address = address_inputs(builder);
        let data_in = data_inputs(builder);
        let cells = array_cells(builder);
        let flags = if one_time {
            Some(flag_cells(builder))
        } else {
            None
        };
        rom_load(builder, &cells, flags.as_deref(), load, &address, &data_in);
        let data_out = array_read(builder, &cells, &address);
        for (index, net) in data_out.iter().enumerate() {
            builder.output(DATA_OUT[index], *net);
        }
        (ROM_INPUTS.to_vec(), DATA_OUT.to_vec())
    })
}

fn flag_cells(builder: &mut DigitalBuilder) -> Vec<NetId> {
    (0..WORDS).map(|_| builder.net()).collect()
}

fn rom_load(
    builder: &mut DigitalBuilder,
    cells: &[Vec<NetId>],
    flags: Option<&[NetId]>,
    load: NetId,
    address: &[NetId; ADDRESS_BITS],
    data_in: &[NetId; DATA_BITS],
) {
    for (index, word) in cells.iter().enumerate() {
        let selected = equals(builder, address, index as u32);
        let write = builder.and(load, selected);
        match flags {
            Some(flags) => prom_word(builder, word, flags[index], data_in, write),
            None => rom_word(builder, word, data_in, write),
        }
    }
}

fn prom_word(
    builder: &mut DigitalBuilder,
    word: &[NetId],
    flag: NetId,
    data_in: &[NetId; DATA_BITS],
    write: NetId,
) {
    let not_set = builder.not(flag);
    let accepted = builder.and(write, not_set);
    rom_word(builder, word, data_in, accepted);
    let next = builder.or(flag, accepted);
    builder.dff_into(next, flag);
}

fn rom_word(
    builder: &mut DigitalBuilder,
    word: &[NetId],
    data_in: &[NetId; DATA_BITS],
    write: NetId,
) {
    for bit in 0..DATA_BITS {
        let next = builder.mux(word[bit], data_in[bit], write);
        builder.dff_into(next, word[bit]);
    }
}

const MODULE_INPUTS: [&str; 19] = [
    "reset", "cs", "we", "oe", "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6",
    "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7",
];
const MODULE_OUTPUTS: [&str; 9] = [
    "dout0", "dout1", "dout2", "dout3", "dout4", "dout5", "dout6", "dout7", "z",
];
const CONTROLLER_INPUTS: [&str; 18] = [
    "reset", "req", "we", "addr0", "addr1", "addr2", "addr3", "addr4", "addr5", "addr6", "din0",
    "din1", "din2", "din3", "din4", "din5", "din6", "din7",
];
const CONTROLLER_OUTPUTS: [&str; 10] = [
    "dout0", "dout1", "dout2", "dout3", "dout4", "dout5", "dout6", "dout7", "ready", "valid",
];

#[derive(Clone, Copy)]
struct ControllerInputs {
    reset: NetId,
    req: NetId,
    we: NetId,
    address: [NetId; ADDRESS_BITS],
    data_in: [NetId; DATA_BITS],
}

#[derive(Clone, Copy)]
struct ControllerState {
    busy: NetId,
    write_op: NetId,
    output: [NetId; DATA_BITS],
}

pub(crate) fn memory_module_entry(id: &'static str) -> DigitalEntry {
    finish(id, |builder| {
        let reset = builder.input("reset");
        let cs = builder.input("cs");
        let we = builder.input("we");
        let oe = builder.input("oe");
        let address = address_inputs(builder);
        let data_in = data_inputs(builder);
        let cells = array_cells(builder);
        let write = builder.and(cs, we);
        sram_write(builder, &cells, reset, write, false, &address, &data_in);
        module_outputs(builder, cs, we, oe, &cells, &address);
        (MODULE_INPUTS.to_vec(), MODULE_OUTPUTS.to_vec())
    })
}

fn module_outputs(
    builder: &mut DigitalBuilder,
    cs: NetId,
    we: NetId,
    oe: NetId,
    cells: &[Vec<NetId>],
    address: &[NetId; ADDRESS_BITS],
) {
    let data = array_read(builder, cells, address);
    let not_we = builder.not(we);
    let selected = builder.and(cs, oe);
    let enabled = builder.and(selected, not_we);
    let not_enabled = builder.not(enabled);
    for (index, net) in data.iter().enumerate() {
        let value = builder.and(*net, enabled);
        builder.output(MODULE_OUTPUTS[index], value);
    }
    builder.output("z", not_enabled);
}

pub(crate) fn memory_controller_entry(id: &'static str) -> DigitalEntry {
    finish(id, |builder| {
        let inputs = controller_inputs(builder);
        let state = controller_state_nets(builder);
        let cells = array_cells(builder);
        controller_state(builder, inputs, state, &cells);
        controller_outputs(builder, state);
        (CONTROLLER_INPUTS.to_vec(), CONTROLLER_OUTPUTS.to_vec())
    })
}

fn controller_inputs(builder: &mut DigitalBuilder) -> ControllerInputs {
    ControllerInputs {
        reset: builder.input("reset"),
        req: builder.input("req"),
        we: builder.input("we"),
        address: address_inputs(builder),
        data_in: data_inputs(builder),
    }
}

fn controller_state_nets(builder: &mut DigitalBuilder) -> ControllerState {
    ControllerState {
        busy: builder.net(),
        write_op: builder.net(),
        output: std::array::from_fn(|_| builder.net()),
    }
}

#[derive(Clone, Copy)]
struct ControllerNext {
    busy: NetId,
    write_op: NetId,
    output: [NetId; DATA_BITS],
    write: NetId,
}

fn controller_state(
    builder: &mut DigitalBuilder,
    inputs: ControllerInputs,
    state: ControllerState,
    cells: &[Vec<NetId>],
) {
    let next = controller_next(builder, inputs, state, cells);
    controller_store(builder, state, next);
    sram_write(
        builder,
        cells,
        inputs.reset,
        next.write,
        false,
        &inputs.address,
        &inputs.data_in,
    );
}

fn controller_next(
    builder: &mut DigitalBuilder,
    inputs: ControllerInputs,
    state: ControllerState,
    cells: &[Vec<NetId>],
) -> ControllerNext {
    let not_busy = builder.not(state.busy);
    let accept = builder.and(not_busy, inputs.req);
    let write = builder.and(accept, inputs.we);
    let read = array_read(builder, cells, &inputs.address);
    let busy = controller_busy(builder, accept, inputs.reset);
    let write_op = write_value(builder, inputs.reset, accept, inputs.we, state.write_op);
    let output = output_value(builder, inputs.reset, accept, &read, &state.output);
    ControllerNext {
        busy,
        write_op,
        output,
        write,
    }
}

fn controller_busy(builder: &mut DigitalBuilder, accept: NetId, reset: NetId) -> NetId {
    let not_reset = builder.not(reset);
    builder.and(accept, not_reset)
}

fn controller_store(builder: &mut DigitalBuilder, state: ControllerState, next: ControllerNext) {
    let one = builder.constant(true);
    builder.dff_into(next.busy, state.busy);
    builder.dff_into(next.write_op, state.write_op);
    register_bits(builder, &state.output, &next.output, one);
}

fn write_value(
    builder: &mut DigitalBuilder,
    reset: NetId,
    accept: NetId,
    we: NetId,
    hold: NetId,
) -> NetId {
    let zero = builder.constant(false);
    let value = builder.mux(hold, we, accept);
    builder.mux(value, zero, reset)
}

fn output_value(
    builder: &mut DigitalBuilder,
    reset: NetId,
    accept: NetId,
    read: &[NetId; DATA_BITS],
    hold: &[NetId; DATA_BITS],
) -> [NetId; DATA_BITS] {
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(hold[index], read[index], accept);
        builder.mux(value, zero, reset)
    })
}

fn controller_outputs(builder: &mut DigitalBuilder, state: ControllerState) {
    let not_write = builder.not(state.write_op);
    let valid = builder.and(state.busy, not_write);
    for (index, net) in state.output.iter().enumerate() {
        builder.output(CONTROLLER_OUTPUTS[index], *net);
    }
    builder.output("ready", state.busy);
    builder.output("valid", valid);
}

const DRAM_INPUTS: [&str; 4] = ["reset", "refresh", "we", "d"];
const DRAM_OUTPUTS: [&str; 2] = ["q", "expired"];

pub(crate) fn dram_cell_entry(id: &'static str) -> DigitalEntry {
    finish(id, |builder| {
        let reset = builder.input("reset");
        let refresh = builder.input("refresh");
        let we = builder.input("we");
        let data = builder.input("d");
        let q = builder.net();
        let counter = [builder.net(), builder.net(), builder.net(), builder.net()];
        dram_registers(builder, reset, refresh, we, data, q, counter);
        let expired = equals(builder, &counter, 15);
        builder.output("q", q);
        builder.output("expired", expired);
        (DRAM_INPUTS.to_vec(), DRAM_OUTPUTS.to_vec())
    })
}

fn dram_registers(
    builder: &mut DigitalBuilder,
    reset: NetId,
    refresh: NetId,
    we: NetId,
    data: NetId,
    q: NetId,
    counter: [NetId; 4],
) {
    let expired = equals(builder, &counter, 15);
    let q_next = dram_q_next(builder, reset, refresh, we, data, q, expired);
    let counter_next = dram_counter_next(builder, reset, refresh, we, &counter, expired);
    let one = builder.constant(true);
    builder.dff_into(q_next, q);
    register_bits(builder, &counter, &counter_next, one);
}

fn dram_q_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    refresh: NetId,
    we: NetId,
    data: NetId,
    q: NetId,
    expired: NetId,
) -> NetId {
    let zero = builder.constant(false);
    let value = builder.mux(q, zero, expired);
    let value = builder.mux(value, q, refresh);
    let value = builder.mux(value, data, we);
    builder.mux(value, zero, reset)
}

fn dram_counter_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    refresh: NetId,
    we: NetId,
    counter: &[NetId; 4],
    expired: NetId,
) -> [NetId; 4] {
    let incremented = increment(builder, counter);
    let zero = builder.constant(false);
    std::array::from_fn(|index| {
        let value = builder.mux(incremented[index], counter[index], expired);
        let value = builder.mux(value, zero, refresh);
        let value = builder.mux(value, zero, we);
        builder.mux(value, zero, reset)
    })
}

pub(crate) fn sram_cell_entry(id: &'static str) -> DigitalEntry {
    finish(id, |builder| {
        let reset = builder.input("reset");
        let write_enable = builder.input("we");
        let data = builder.input("d");
        let output = builder.net();
        let next = builder.mux(output, data, write_enable);
        let not_reset = builder.not(reset);
        let next = builder.and(next, not_reset);
        builder.dff_into(next, output);
        builder.output("q", output);
        (vec!["reset", "we", "d"], vec!["q"])
    })
}

fn sram_inputs() -> Vec<&'static str> {
    let mut inputs = vec!["reset", "we"];
    inputs.extend_from_slice(&ADDRESS);
    inputs.extend_from_slice(&DATA_IN);
    inputs
}

fn address_inputs(builder: &mut DigitalBuilder) -> [NetId; ADDRESS_BITS] {
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

fn data_inputs(builder: &mut DigitalBuilder) -> [NetId; DATA_BITS] {
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

pub(crate) fn array_cells(builder: &mut DigitalBuilder) -> Vec<Vec<NetId>> {
    array_cells_n(builder, WORDS)
}

pub(crate) fn array_cells_n(builder: &mut DigitalBuilder, words: usize) -> Vec<Vec<NetId>> {
    let mut cells = Vec::with_capacity(words);
    for _ in 0..words {
        let mut word = Vec::with_capacity(DATA_BITS);
        for _ in 0..DATA_BITS {
            word.push(builder.net());
        }
        cells.push(word);
    }
    cells
}

fn sram_write(
    builder: &mut DigitalBuilder,
    cells: &[Vec<NetId>],
    reset: NetId,
    write_enable: NetId,
    retain: bool,
    address: &[NetId; ADDRESS_BITS],
    data_in: &[NetId; DATA_BITS],
) {
    let not_reset = builder.not(reset);
    for (index, word) in cells.iter().enumerate() {
        let selected = equals(builder, address, index as u32);
        let write = builder.and(write_enable, selected);
        write_word(builder, word, data_in, write, retain, not_reset);
    }
}

fn write_word(
    builder: &mut DigitalBuilder,
    word: &[NetId],
    data_in: &[NetId; DATA_BITS],
    write: NetId,
    retain: bool,
    not_reset: NetId,
) {
    for bit in 0..DATA_BITS {
        let next = builder.mux(word[bit], data_in[bit], write);
        let next = if retain {
            next
        } else {
            builder.and(next, not_reset)
        };
        builder.dff_into(next, word[bit]);
    }
}

pub(crate) fn array_read(
    builder: &mut DigitalBuilder,
    cells: &[Vec<NetId>],
    address: &[NetId; ADDRESS_BITS],
) -> [NetId; DATA_BITS] {
    array_read_n(builder, cells, address)
}

pub(crate) fn array_read_n(
    builder: &mut DigitalBuilder,
    cells: &[Vec<NetId>],
    address: &[NetId],
) -> [NetId; DATA_BITS] {
    std::array::from_fn(|bit| {
        let column: Vec<NetId> = cells.iter().map(|word| word[bit]).collect();
        mux_tree(builder, column, address)
    })
}

#[cfg(test)]
mod tests {
    use super::{DATA_BITS, memory_array_entry, sram_cell_entry};

    fn set_address(entry: &mut super::DigitalEntry, value: usize) {
        for index in 0..7 {
            let bit = value & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("addr{index}"), bit)
                .expect("address");
        }
    }

    fn set_data(entry: &mut super::DigitalEntry, value: u8) {
        for index in 0..DATA_BITS {
            let bit = value & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("din{index}"), bit)
                .expect("data");
        }
    }

    fn read_data(entry: &super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..DATA_BITS {
            if entry.circuit.output(&format!("dout{index}")) == Some(true) {
                value |= 1 << index;
            }
        }
        value
    }

    fn write_word(entry: &mut super::DigitalEntry, address: usize, value: u8) {
        set_address(entry, address);
        set_data(entry, value);
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.tick().expect("write");
        entry.circuit.set_input("we", false).expect("we");
    }

    fn read_word(entry: &mut super::DigitalEntry, address: usize) -> u8 {
        set_address(entry, address);
        entry.circuit.settle().expect("read");
        read_data(entry)
    }

    #[test]
    fn sram_array_stores_and_reads_words() {
        let mut entry = memory_array_entry("memory.device.sram_array", false);
        write_word(&mut entry, 5, 0xa5);
        write_word(&mut entry, 100, 0x3c);
        assert_eq!(read_word(&mut entry, 5), 0xa5);
        assert_eq!(read_word(&mut entry, 100), 0x3c);
    }

    #[test]
    fn sram_array_reset_clears_words() {
        let mut entry = memory_array_entry("memory.device.sram_array", false);
        write_word(&mut entry, 5, 0xa5);
        entry.circuit.set_input("reset", true).expect("reset");
        entry.circuit.tick().expect("reset");
        entry.circuit.set_input("reset", false).expect("reset");
        assert_eq!(read_word(&mut entry, 5), 0);
    }

    fn load_word(entry: &mut super::DigitalEntry, address: usize, value: u8) {
        set_address(entry, address);
        set_data(entry, value);
        entry.circuit.set_input("load", true).expect("load");
        entry.circuit.tick().expect("load");
        entry.circuit.set_input("load", false).expect("load");
    }

    #[test]
    fn rom_reads_loaded_word() {
        let mut entry = super::rom_array_entry("memory.device.rom");
        load_word(&mut entry, 9, 0x77);
        assert_eq!(read_word(&mut entry, 9), 0x77);
    }

    #[test]
    fn prom_ignores_second_program() {
        let mut entry = super::prom_array_entry("memory.device.prom");
        load_word(&mut entry, 9, 0x77);
        load_word(&mut entry, 9, 0x11);
        assert_eq!(read_word(&mut entry, 9), 0x77);
    }

    #[test]
    fn memory_module_respects_chip_select() {
        let mut entry = super::memory_module_entry("memory.device.memory_module");
        set_address(&mut entry, 3);
        set_data(&mut entry, 0x99);
        entry.circuit.set_input("cs", true).expect("cs");
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.tick().expect("write");
        entry.circuit.set_input("we", false).expect("we");
        entry.circuit.set_input("oe", true).expect("oe");
        entry.circuit.settle().expect("read");
        assert_eq!(read_data(&entry), 0x99);
        assert_eq!(entry.circuit.output("z"), Some(false));
        entry.circuit.set_input("cs", false).expect("cs");
        entry.circuit.settle().expect("select off");
        assert_eq!(entry.circuit.output("z"), Some(true));
        assert_eq!(read_data(&entry), 0);
    }

    #[test]
    fn memory_controller_handshakes_write_and_read() {
        let mut entry = super::memory_controller_entry("memory.device.memory_controller");
        set_address(&mut entry, 6);
        set_data(&mut entry, 0x3c);
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.set_input("req", true).expect("req");
        entry.circuit.tick().expect("write");
        assert_eq!(entry.circuit.output("ready"), Some(true));
        entry.circuit.set_input("req", false).expect("req");
        entry.circuit.tick().expect("idle");
        entry.circuit.set_input("we", false).expect("we");
        entry.circuit.set_input("req", true).expect("req");
        entry.circuit.tick().expect("read");
        assert_eq!(entry.circuit.output("ready"), Some(true));
        assert_eq!(entry.circuit.output("valid"), Some(true));
        assert_eq!(read_data(&entry), 0x3c);
    }

    #[test]
    fn non_volatile_array_retains_after_reset() {
        let mut entry = memory_array_entry("memory.device.fram", true);
        write_word(&mut entry, 7, 0x5a);
        entry.circuit.set_input("reset", true).expect("reset");
        entry.circuit.tick().expect("reset");
        entry.circuit.set_input("reset", false).expect("reset");
        assert_eq!(read_word(&mut entry, 7), 0x5a);
    }

    #[test]
    fn dram_cell_expires_without_refresh() {
        let mut entry = super::dram_cell_entry("memory.device.dram_cell");
        entry.circuit.set_input("d", true).expect("d");
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.tick().expect("write");
        entry.circuit.set_input("we", false).expect("we");
        assert_eq!(entry.circuit.output("q"), Some(true));
        for _ in 0..15 {
            entry.circuit.tick().expect("retain");
        }
        assert_eq!(entry.circuit.output("q"), Some(true));
        entry.circuit.tick().expect("expire");
        assert_eq!(entry.circuit.output("q"), Some(false));
        assert_eq!(entry.circuit.output("expired"), Some(true));
    }

    #[test]
    fn dram_cell_retains_with_refresh() {
        let mut entry = super::dram_cell_entry("memory.device.dram_cell");
        entry.circuit.set_input("d", true).expect("d");
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.tick().expect("write");
        entry.circuit.set_input("we", false).expect("we");
        for _ in 0..10 {
            entry.circuit.tick().expect("retain");
        }
        entry.circuit.set_input("refresh", true).expect("refresh");
        entry.circuit.tick().expect("refresh");
        entry.circuit.set_input("refresh", false).expect("refresh");
        for _ in 0..10 {
            entry.circuit.tick().expect("retain");
        }
        assert_eq!(entry.circuit.output("q"), Some(true));
    }

    #[test]
    fn sram_cell_holds_when_write_disabled() {
        let mut entry = sram_cell_entry("memory.device.sram_cell");
        entry.circuit.set_input("d", true).expect("d");
        entry.circuit.set_input("we", true).expect("we");
        entry.circuit.tick().expect("tick");
        assert_eq!(entry.circuit.output("q"), Some(true));
        entry.circuit.set_input("d", false).expect("d");
        entry.circuit.set_input("we", false).expect("we");
        entry.circuit.tick().expect("tick");
        assert_eq!(entry.circuit.output("q"), Some(true));
    }
}
