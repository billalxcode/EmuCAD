//! Digital register file with two read ports.

use super::blocks::{decoder, mux_tree, register, register_bit};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Wiring for a register file.
pub struct RegisterPorts {
    /// Write address nets, least significant bit first.
    pub write_address: Vec<NetId>,
    /// Write data nets, least significant bit first.
    pub write_data: Vec<NetId>,
    /// Write enable.
    pub write_enable: NetId,
    /// Read address A nets.
    pub read_address_a: Vec<NetId>,
    /// Read address B nets.
    pub read_address_b: Vec<NetId>,
}

/// A two-read-port register file built from D flip-flops.
pub struct RegisterFile {
    bits: Vec<Vec<NetId>>,
    read_a: Vec<NetId>,
    read_b: Vec<NetId>,
}

impl RegisterFile {
    /// Returns storage bit nets indexed by register and bit.
    #[must_use]
    pub fn bits(&self) -> &[Vec<NetId>] {
        &self.bits
    }

    /// Returns read port A nets.
    #[must_use]
    pub fn read_a(&self) -> &[NetId] {
        &self.read_a
    }

    /// Returns read port B nets.
    #[must_use]
    pub fn read_b(&self) -> &[NetId] {
        &self.read_b
    }
}

/// Builds a register file.
pub fn register_file(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
) -> Result<RegisterFile, SimError> {
    validate_shape(count, width, ports)?;
    let bits = build_storage(builder, count, width, ports)?;
    let read_a = optional_read_port(builder, &bits, width, &ports.read_address_a)?;
    let read_b = optional_read_port(builder, &bits, width, &ports.read_address_b)?;
    Ok(RegisterFile {
        bits,
        read_a,
        read_b,
    })
}

/// Builds a register file with byte write masks.
pub fn register_file_masked(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
    mask: &[NetId],
) -> Result<RegisterFile, SimError> {
    validate_shape(count, width, ports)?;
    validate_mask(width, mask)?;
    let bits = build_masked_storage(builder, count, width, ports, mask)?;
    let read_a = optional_read_port(builder, &bits, width, &ports.read_address_a)?;
    let read_b = optional_read_port(builder, &bits, width, &ports.read_address_b)?;
    Ok(RegisterFile {
        bits,
        read_a,
        read_b,
    })
}

fn validate_mask(width: usize, mask: &[NetId]) -> Result<(), SimError> {
    if mask.len() != width.div_ceil(8) {
        return Err(SimError::InvalidNetlist(
            "register mask must have one bit per byte".to_string(),
        ));
    }
    Ok(())
}

fn build_masked_storage(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
    mask: &[NetId],
) -> Result<Vec<Vec<NetId>>, SimError> {
    let lines = decoder(builder, &ports.write_address)?;
    let mut bits = Vec::with_capacity(count);
    for line in lines {
        let enable = builder.and(line, ports.write_enable);
        bits.push(masked_row(builder, enable, &ports.write_data, mask, width));
    }
    Ok(bits)
}

fn masked_row(
    builder: &mut DigitalBuilder,
    enable: NetId,
    data: &[NetId],
    mask: &[NetId],
    width: usize,
) -> Vec<NetId> {
    (0..width)
        .map(|bit| {
            let load = builder.and(enable, mask[bit / 8]);
            register_bit(builder, data[bit], load)
        })
        .collect()
}

fn optional_read_port(
    builder: &mut DigitalBuilder,
    bits: &[Vec<NetId>],
    width: usize,
    address: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    if address.is_empty() {
        return Ok(Vec::new());
    }
    build_read_port(builder, bits, width, address)
}

fn validate_shape(count: usize, width: usize, ports: &RegisterPorts) -> Result<(), SimError> {
    validate_count(count)?;
    validate_width(width)?;
    validate_address(count, &ports.write_address)?;
    if !ports.read_address_a.is_empty() {
        validate_address(count, &ports.read_address_a)?;
    }
    if !ports.read_address_b.is_empty() {
        validate_address(count, &ports.read_address_b)?;
    }
    validate_data(width, &ports.write_data)
}

fn validate_count(count: usize) -> Result<(), SimError> {
    if !count.is_power_of_two() || !(2..=32).contains(&count) {
        return Err(SimError::InvalidNetlist(
            "register file count must be a power of two in 2..=32".to_string(),
        ));
    }
    Ok(())
}

fn validate_width(width: usize) -> Result<(), SimError> {
    if !(1..=64).contains(&width) {
        return Err(SimError::InvalidNetlist(
            "register width must be 1..=64".to_string(),
        ));
    }
    Ok(())
}

fn validate_address(count: usize, address: &[NetId]) -> Result<(), SimError> {
    if address.len() != count.trailing_zeros() as usize {
        return Err(SimError::InvalidNetlist(
            "register address width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_data(width: usize, data: &[NetId]) -> Result<(), SimError> {
    if data.len() != width {
        return Err(SimError::InvalidNetlist(
            "register write data width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn build_storage(
    builder: &mut DigitalBuilder,
    count: usize,
    width: usize,
    ports: &RegisterPorts,
) -> Result<Vec<Vec<NetId>>, SimError> {
    let lines = decoder(builder, &ports.write_address)?;
    let mut bits = Vec::with_capacity(count);
    for line in lines {
        let enable = builder.and(line, ports.write_enable);
        bits.push(register(builder, width, &ports.write_data, enable)?);
    }
    Ok(bits)
}

fn build_read_port(
    builder: &mut DigitalBuilder,
    bits: &[Vec<NetId>],
    width: usize,
    address: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    (0..width)
        .map(|bit| {
            let column: Vec<NetId> = bits.iter().map(|register| register[bit]).collect();
            mux_tree(builder, &column, address)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{RegisterPorts, register_file, register_file_masked};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};

    const FILE_COUNT: usize = 32;
    const ADDRESS_WIDTH: usize = 5;
    const BYTE_WIDTH: usize = 8;
    const WORD_WIDTH: usize = 32;
    const MASK_WIDTH: usize = 4;

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn expose(builder: &mut DigitalBuilder, prefix: &str, nets: &[NetId]) {
        for (index, &net) in nets.iter().enumerate() {
            builder.output(&format!("{prefix}{index}"), net);
        }
    }

    fn fixture() -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let ports = fixture_ports(&mut builder, BYTE_WIDTH);
        let file = register_file(&mut builder, FILE_COUNT, BYTE_WIDTH, &ports).expect("file");
        expose(&mut builder, "a", file.read_a());
        expose(&mut builder, "b", file.read_b());
        builder.build()
    }

    fn masked_fixture() -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let ports = fixture_ports(&mut builder, WORD_WIDTH);
        let mask = named_inputs(&mut builder, "msk", MASK_WIDTH);
        let file = register_file_masked(&mut builder, FILE_COUNT, WORD_WIDTH, &ports, &mask)
            .expect("file");
        expose(&mut builder, "a", file.read_a());
        expose(&mut builder, "b", file.read_b());
        builder.build()
    }

    fn fixture_ports(builder: &mut DigitalBuilder, width: usize) -> RegisterPorts {
        RegisterPorts {
            write_address: named_inputs(builder, "wa", ADDRESS_WIDTH),
            write_data: named_inputs(builder, "wd", width),
            write_enable: builder.input("we"),
            read_address_a: named_inputs(builder, "ra", ADDRESS_WIDTH),
            read_address_b: named_inputs(builder, "rb", ADDRESS_WIDTH),
        }
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn write_register(circuit: &mut DigitalCircuit, address: usize, value: u64, width: usize) {
        set_bits(circuit, "wa", address as u64, ADDRESS_WIDTH);
        set_bits(circuit, "wd", value, width);
        circuit.set_input("we", true).expect("we");
        circuit.tick().expect("tick");
    }

    fn write_masked(circuit: &mut DigitalCircuit, address: usize, value: u64, mask: u64) {
        set_bits(circuit, "msk", mask, MASK_WIDTH);
        write_register(circuit, address, value, WORD_WIDTH);
    }

    fn read_port(
        circuit: &mut DigitalCircuit,
        input_prefix: &str,
        output_prefix: &str,
        address: usize,
        width: usize,
    ) -> u64 {
        set_bits(circuit, input_prefix, address as u64, ADDRESS_WIDTH);
        circuit.settle().expect("settle");
        read_named(circuit, output_prefix, width)
    }

    fn read_named(circuit: &DigitalCircuit, prefix: &str, width: usize) -> u64 {
        let mut value = 0;
        for index in 0..width {
            if circuit.output(&format!("{prefix}{index}")).expect("output") {
                value |= 1 << index;
            }
        }
        value
    }

    fn sample_value(address: usize) -> u64 {
        (address as u64 * 7 + 3) & 0xFF
    }

    #[test]
    fn register_file_writes_and_reads_all_registers() {
        let mut circuit = fixture();
        for address in 0..FILE_COUNT {
            write_register(&mut circuit, address, sample_value(address), BYTE_WIDTH);
        }
        for address in 0..FILE_COUNT {
            let expected = sample_value(address);
            assert_eq!(
                read_port(&mut circuit, "ra", "a", address, BYTE_WIDTH),
                expected
            );
        }
    }

    #[test]
    fn register_file_holds_without_write() {
        let mut circuit = fixture();
        write_register(&mut circuit, 1, 0x55, BYTE_WIDTH);
        set_bits(&mut circuit, "wa", 1, ADDRESS_WIDTH);
        set_bits(&mut circuit, "wd", 0x30, BYTE_WIDTH);
        circuit.set_input("we", false).expect("we");
        circuit.tick().expect("tick");
        assert_eq!(read_port(&mut circuit, "ra", "a", 1, BYTE_WIDTH), 0x55);
    }

    #[test]
    fn register_file_ports_are_independent() {
        let mut circuit = fixture();
        write_register(&mut circuit, 0, 0x3, BYTE_WIDTH);
        write_register(&mut circuit, 31, 0xC, BYTE_WIDTH);
        assert_eq!(read_port(&mut circuit, "ra", "a", 0, BYTE_WIDTH), 0x3);
        assert_eq!(read_port(&mut circuit, "rb", "b", 31, BYTE_WIDTH), 0xC);
    }

    #[test]
    fn register_file_reset_clears_storage() {
        let mut circuit = fixture();
        write_register(&mut circuit, 2, 0xF, BYTE_WIDTH);
        circuit.reset().expect("reset");
        assert_eq!(read_port(&mut circuit, "ra", "a", 2, BYTE_WIDTH), 0);
    }

    #[test]
    fn masked_register_file_writes_32_bit_words() {
        let mut circuit = masked_fixture();
        write_masked(&mut circuit, 5, 0xDEAD_BEEF, 0xF);
        write_masked(&mut circuit, 31, 0x1234_5678, 0xF);
        assert_eq!(
            read_port(&mut circuit, "ra", "a", 5, WORD_WIDTH),
            0xDEAD_BEEF
        );
        assert_eq!(
            read_port(&mut circuit, "rb", "b", 31, WORD_WIDTH),
            0x1234_5678
        );
    }

    #[test]
    fn masked_register_file_retains_unmasked_bytes() {
        let mut circuit = masked_fixture();
        write_masked(&mut circuit, 7, 0x1122_3344, 0xF);
        write_masked(&mut circuit, 7, 0x0000_AA00, 0b0010);
        assert_eq!(
            read_port(&mut circuit, "ra", "a", 7, WORD_WIDTH),
            0x1122_AA44
        );
        write_masked(&mut circuit, 7, 0xFF00_0000, 0b1000);
        assert_eq!(
            read_port(&mut circuit, "rb", "b", 7, WORD_WIDTH),
            0xFF22_AA44
        );
        write_masked(&mut circuit, 7, 0xFFFF_FFFF, 0b0000);
        assert_eq!(
            read_port(&mut circuit, "ra", "a", 7, WORD_WIDTH),
            0xFF22_AA44
        );
    }

    #[test]
    fn masked_register_file_rejects_wrong_mask_width() {
        let mut builder = DigitalBuilder::new();
        let ports = fixture_ports(&mut builder, WORD_WIDTH);
        let mask = named_inputs(&mut builder, "msk", MASK_WIDTH - 1);
        let result = register_file_masked(&mut builder, FILE_COUNT, WORD_WIDTH, &ports, &mask);
        assert!(result.is_err());
    }
}
