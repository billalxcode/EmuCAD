//! Gate-level SRAM cell and digital memory arrays.

use crate::catalog_bits::{equals, mux_tree};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const ADDRESS_BITS: usize = 7;
const DATA_BITS: usize = 8;
const WORDS: usize = 1 << ADDRESS_BITS;
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
        let cells = sram_cells(builder);
        sram_write(
            builder,
            &cells,
            reset,
            write_enable,
            retain,
            &address,
            &data_in,
        );
        let data_out = sram_read(builder, &cells, &address);
        for (index, net) in data_out.iter().enumerate() {
            builder.output(DATA_OUT[index], *net);
        }
        (sram_inputs(), DATA_OUT.to_vec())
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

fn sram_cells(builder: &mut DigitalBuilder) -> Vec<Vec<NetId>> {
    let mut cells = Vec::with_capacity(WORDS);
    for _ in 0..WORDS {
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

fn sram_read(
    builder: &mut DigitalBuilder,
    cells: &[Vec<NetId>],
    address: &[NetId; ADDRESS_BITS],
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
