//! Parameterized synchronous memory.

use super::blocks::register;
use super::circuit::{DigitalBuilder, NetId};
use super::counters::counter_match;
use super::registers::{RegisterFile, RegisterPorts, register_file, register_file_masked};
use kernel::SimError;

/// Read behavior when a write and read target the same address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadDuringWrite {
    /// Read returns the stored value from before the write.
    ReadFirst,
    /// Read returns the write data for the same cycle.
    WriteFirst,
}

/// Shape and timing of a synchronous memory.
pub struct MemoryConfig {
    /// Number of words, a power of two in 2..=32.
    pub depth: usize,
    /// Word width in bits, 1..=64.
    pub width: usize,
    /// Read latency: 0 combinational or 1 registered.
    pub latency: usize,
    /// Read-during-write policy.
    pub read_during_write: ReadDuringWrite,
}

/// Ports of a synchronous memory.
pub struct MemoryPorts {
    /// Write address, least significant bit first.
    pub write_address: Vec<NetId>,
    /// Write data, least significant bit first.
    pub write_data: Vec<NetId>,
    /// Write enable.
    pub write_enable: NetId,
    /// Optional byte write mask; an empty slice writes every byte.
    pub write_mask: Vec<NetId>,
    /// Read address, least significant bit first.
    pub read_address: Vec<NetId>,
    /// Read enable; registered reads hold when low.
    pub read_enable: NetId,
}

/// Built synchronous memory.
pub struct Memory {
    /// Read data, least significant bit first.
    pub read_data: Vec<NetId>,
}

/// Builds a synchronous single-clock memory.
///
/// Storage resets to zero. Read-during-write defaults are selected by
/// [`ReadDuringWrite`].
pub fn memory_sync(
    builder: &mut DigitalBuilder,
    config: &MemoryConfig,
    ports: &MemoryPorts,
) -> Result<Memory, SimError> {
    validate_memory(config, ports)?;
    let file = memory_storage(builder, config, ports)?;
    let raw = file.read_a().to_vec();
    let data = memory_read_data(builder, config, ports, &raw)?;
    let read_data = if config.latency == 1 {
        register(builder, config.width, &data, ports.read_enable)?
    } else {
        data
    };
    Ok(Memory { read_data })
}

fn memory_storage(
    builder: &mut DigitalBuilder,
    config: &MemoryConfig,
    ports: &MemoryPorts,
) -> Result<RegisterFile, SimError> {
    let register_ports = memory_register_ports(ports);
    if ports.write_mask.is_empty() {
        return register_file(builder, config.depth, config.width, &register_ports);
    }
    register_file_masked(
        builder,
        config.depth,
        config.width,
        &register_ports,
        &ports.write_mask,
    )
}

fn memory_register_ports(ports: &MemoryPorts) -> RegisterPorts {
    RegisterPorts {
        write_address: ports.write_address.clone(),
        write_data: ports.write_data.clone(),
        write_enable: ports.write_enable,
        read_address_a: ports.read_address.clone(),
        read_address_b: Vec::new(),
    }
}

fn memory_read_data(
    builder: &mut DigitalBuilder,
    config: &MemoryConfig,
    ports: &MemoryPorts,
    raw: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    if config.read_during_write == ReadDuringWrite::ReadFirst {
        return Ok(raw.to_vec());
    }
    let same = counter_match(builder, &ports.write_address, &ports.read_address)?;
    Ok((0..config.width)
        .map(|bit| memory_bypass_bit(builder, ports, raw[bit], ports.write_data[bit], bit, same))
        .collect())
}

fn memory_bypass_bit(
    builder: &mut DigitalBuilder,
    ports: &MemoryPorts,
    raw: NetId,
    data: NetId,
    bit: usize,
    same: NetId,
) -> NetId {
    let enabled = if ports.write_mask.is_empty() {
        ports.write_enable
    } else {
        builder.and(ports.write_enable, ports.write_mask[bit / 8])
    };
    let use_write = builder.and(same, enabled);
    builder.mux(raw, data, use_write)
}

fn validate_memory(config: &MemoryConfig, ports: &MemoryPorts) -> Result<(), SimError> {
    validate_memory_shape(config)?;
    validate_memory_ports(config, ports)
}

fn validate_memory_shape(config: &MemoryConfig) -> Result<(), SimError> {
    if !(2..=32).contains(&config.depth) || !config.depth.is_power_of_two() {
        return Err(invalid("memory depth must be a power of two in 2..=32"));
    }
    if config.width == 0 || config.width > 64 {
        return Err(invalid("memory width must be 1..=64"));
    }
    if config.latency > 1 {
        return Err(invalid("memory latency must be 0 or 1"));
    }
    Ok(())
}

fn validate_memory_ports(config: &MemoryConfig, ports: &MemoryPorts) -> Result<(), SimError> {
    let address_width = config.depth.trailing_zeros() as usize;
    if ports.write_address.len() != address_width || ports.read_address.len() != address_width {
        return Err(invalid("memory address width mismatch"));
    }
    if ports.write_data.len() != config.width {
        return Err(invalid("memory write data width mismatch"));
    }
    if !ports.write_mask.is_empty() && ports.write_mask.len() != config.width.div_ceil(8) {
        return Err(invalid("memory mask must have one bit per byte"));
    }
    Ok(())
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Memory, MemoryConfig, MemoryPorts, ReadDuringWrite, memory_sync};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn simple_memory(latency: usize, policy: ReadDuringWrite) -> (DigitalCircuit, Memory) {
        let mut builder = DigitalBuilder::new();
        let ports = MemoryPorts {
            write_address: named_inputs(&mut builder, "wa", 2),
            write_data: named_inputs(&mut builder, "wd", 8),
            write_enable: builder.input("we"),
            write_mask: Vec::new(),
            read_address: named_inputs(&mut builder, "ra", 2),
            read_enable: builder.input("re"),
        };
        let config = MemoryConfig {
            depth: 4,
            width: 8,
            latency,
            read_during_write: policy,
        };
        let memory = memory_sync(&mut builder, &config, &ports).expect("memory");
        (builder.build(), memory)
    }

    fn masked_memory() -> (DigitalCircuit, Memory) {
        let mut builder = DigitalBuilder::new();
        let ports = MemoryPorts {
            write_address: named_inputs(&mut builder, "wa", 2),
            write_data: named_inputs(&mut builder, "wd", 16),
            write_enable: builder.input("we"),
            write_mask: named_inputs(&mut builder, "wm", 2),
            read_address: named_inputs(&mut builder, "ra", 2),
            read_enable: builder.input("re"),
        };
        let config = MemoryConfig {
            depth: 4,
            width: 16,
            latency: 0,
            read_during_write: ReadDuringWrite::ReadFirst,
        };
        let memory = memory_sync(&mut builder, &config, &ports).expect("memory");
        (builder.build(), memory)
    }

    fn write_memory(circuit: &mut DigitalCircuit, address: u64, data: u64, width: usize) {
        set_bits(circuit, "wa", address, 2);
        set_bits(circuit, "wd", data, width);
        circuit.set_input("we", true).expect("we");
        circuit.tick().expect("tick");
    }

    fn read_memory(
        circuit: &mut DigitalCircuit,
        memory: &Memory,
        address: u64,
        width: usize,
    ) -> u64 {
        set_bits(circuit, "ra", address, 2);
        circuit.set_input("re", true).expect("re");
        circuit.settle().expect("settle");
        read_bits(circuit, &memory.read_data[..width])
    }

    #[test]
    fn memory_writes_and_reads_words() {
        let (mut circuit, memory) = simple_memory(0, ReadDuringWrite::ReadFirst);
        for address in 0..4 {
            write_memory(&mut circuit, address, address * 3 + 1, 8);
        }
        for address in 0..4 {
            assert_eq!(
                read_memory(&mut circuit, &memory, address, 8),
                address * 3 + 1
            );
        }
    }

    #[test]
    fn memory_read_first_returns_old_value() {
        let (mut circuit, memory) = simple_memory(0, ReadDuringWrite::ReadFirst);
        write_memory(&mut circuit, 0, 0x11, 8);
        set_bits(&mut circuit, "wa", 0, 2);
        set_bits(&mut circuit, "wd", 0xAA, 8);
        set_bits(&mut circuit, "ra", 0, 2);
        circuit.set_input("we", true).expect("we");
        circuit.set_input("re", true).expect("re");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &memory.read_data), 0x11);
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &memory.read_data), 0xAA);
    }

    #[test]
    fn memory_write_first_bypasses_storage() {
        let (mut circuit, memory) = simple_memory(0, ReadDuringWrite::WriteFirst);
        write_memory(&mut circuit, 0, 0x11, 8);
        set_bits(&mut circuit, "wa", 0, 2);
        set_bits(&mut circuit, "wd", 0xBB, 8);
        set_bits(&mut circuit, "ra", 0, 2);
        circuit.set_input("we", true).expect("we");
        circuit.set_input("re", true).expect("re");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &memory.read_data), 0xBB);
    }

    #[test]
    fn registered_read_holds_when_disabled() {
        let (mut circuit, memory) = simple_memory(1, ReadDuringWrite::ReadFirst);
        write_memory(&mut circuit, 1, 0x22, 8);
        set_bits(&mut circuit, "ra", 1, 2);
        circuit.set_input("re", true).expect("re");
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &memory.read_data), 0x22);
        set_bits(&mut circuit, "ra", 0, 2);
        circuit.set_input("re", false).expect("re");
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &memory.read_data), 0x22);
    }

    #[test]
    fn byte_mask_retains_unwritten_bytes() {
        let (mut circuit, memory) = masked_memory();
        set_bits(&mut circuit, "wm", 0b11, 2);
        write_memory(&mut circuit, 0, 0xAAAA, 16);
        set_bits(&mut circuit, "wm", 0b01, 2);
        write_memory(&mut circuit, 0, 0xBBBB, 16);
        assert_eq!(read_memory(&mut circuit, &memory, 0, 16), 0xAABB);
    }

    #[test]
    fn reset_clears_memory() {
        let (mut circuit, memory) = simple_memory(0, ReadDuringWrite::ReadFirst);
        write_memory(&mut circuit, 2, 0x55, 8);
        circuit.reset().expect("reset");
        assert_eq!(read_memory(&mut circuit, &memory, 2, 8), 0);
    }

    fn invalid_memory_ports(builder: &mut DigitalBuilder) -> MemoryPorts {
        MemoryPorts {
            write_address: named_inputs(builder, "wa", 2),
            write_data: named_inputs(builder, "wd", 8),
            write_enable: builder.input("we"),
            write_mask: Vec::new(),
            read_address: named_inputs(builder, "ra", 2),
            read_enable: builder.input("re"),
        }
    }

    fn invalid_test_config() -> MemoryConfig {
        MemoryConfig {
            depth: 3,
            width: 8,
            latency: 0,
            read_during_write: ReadDuringWrite::ReadFirst,
        }
    }

    #[test]
    fn rejects_invalid_memory_shapes() {
        let mut builder = DigitalBuilder::new();
        let ports = invalid_memory_ports(&mut builder);
        let mut config = invalid_test_config();
        assert!(memory_sync(&mut builder, &config, &ports).is_err());
        config.depth = 4;
        config.latency = 2;
        assert!(memory_sync(&mut builder, &config, &ports).is_err());
    }
}
