//! Synchronous FIFO built from a register file and counters.

use super::blocks::{clock_vector, decrement, increment, select_vector};
use super::circuit::{DigitalBuilder, NetId};
use super::counters::{counter_loadable, counter_match};
use super::registers::{RegisterFile, RegisterPorts, register_file};
use kernel::SimError;

/// Ports and status of a synchronous FIFO.
pub struct Fifo {
    /// Current read data, least significant bit first.
    pub read_data: Vec<NetId>,
    /// Storage is full.
    pub full: NetId,
    /// Storage is empty.
    pub empty: NetId,
    /// Number of stored entries.
    pub count: Vec<NetId>,
}

/// Builds a synchronous single-clock FIFO.
pub fn fifo_sync(
    builder: &mut DigitalBuilder,
    width: usize,
    depth: usize,
    write_data: &[NetId],
    write_enable: NetId,
    read_enable: NetId,
) -> Result<Fifo, SimError> {
    validate_fifo(width, depth, write_data)?;
    let inputs = FifoInputs {
        width,
        depth,
        write_data,
        write_enable,
        read_enable,
    };
    let parts = build_fifo_parts(builder, &inputs)?;
    fifo_clock_count(builder, &parts)?;
    Ok(parts.into_fifo())
}

struct FifoInputs<'a> {
    width: usize,
    depth: usize,
    write_data: &'a [NetId],
    write_enable: NetId,
    read_enable: NetId,
}

struct FifoParts {
    count: Vec<NetId>,
    full: NetId,
    empty: NetId,
    file: RegisterFile,
    write_ok: NetId,
    read_ok: NetId,
}

impl FifoParts {
    fn into_fifo(self) -> Fifo {
        Fifo {
            read_data: self.file.read_a().to_vec(),
            full: self.full,
            empty: self.empty,
            count: self.count,
        }
    }
}

fn build_fifo_parts(
    builder: &mut DigitalBuilder,
    inputs: &FifoInputs,
) -> Result<FifoParts, SimError> {
    let zero = builder.constant(false);
    let count = fifo_count_nets(builder, inputs.depth);
    let (full, empty) = fifo_status(builder, &count, inputs.depth, zero)?;
    let write_ok = fifo_enable(builder, inputs.write_enable, full);
    let read_ok = fifo_enable(builder, inputs.read_enable, empty);
    let (write_ptr, read_ptr) = fifo_pointers(builder, inputs.depth, write_ok, read_ok, zero)?;
    let file = fifo_storage(builder, inputs, write_ok, write_ptr, read_ptr)?;
    Ok(FifoParts {
        count,
        full,
        empty,
        file,
        write_ok,
        read_ok,
    })
}

fn fifo_count_nets(builder: &mut DigitalBuilder, depth: usize) -> Vec<NetId> {
    let width = depth.trailing_zeros() as usize + 1;
    (0..width).map(|_| builder.net()).collect()
}

fn fifo_clock_count(builder: &mut DigitalBuilder, parts: &FifoParts) -> Result<(), SimError> {
    let next = fifo_count_next(builder, &parts.count, parts.write_ok, parts.read_ok)?;
    clock_vector(builder, &parts.count, &next);
    Ok(())
}

fn fifo_status(
    builder: &mut DigitalBuilder,
    count: &[NetId],
    depth: usize,
    zero: NetId,
) -> Result<(NetId, NetId), SimError> {
    let target: Vec<NetId> = (0..count.len())
        .map(|bit| builder.constant(depth & (1 << bit) != 0))
        .collect();
    let full = counter_match(builder, count, &target)?;
    let zeroes = vec![zero; count.len()];
    let empty = counter_match(builder, count, &zeroes)?;
    Ok((full, empty))
}

fn fifo_enable(builder: &mut DigitalBuilder, enable: NetId, blocked: NetId) -> NetId {
    let open = builder.not(blocked);
    builder.and(enable, open)
}

fn fifo_pointers(
    builder: &mut DigitalBuilder,
    depth: usize,
    write_ok: NetId,
    read_ok: NetId,
    zero: NetId,
) -> Result<(Vec<NetId>, Vec<NetId>), SimError> {
    let width = depth.trailing_zeros() as usize;
    let zeroes = vec![zero; width];
    let one = builder.constant(true);
    let write_ptr = counter_loadable(builder, width, &zeroes, zero, write_ok, one)?;
    let read_ptr = counter_loadable(builder, width, &zeroes, zero, read_ok, one)?;
    Ok((write_ptr, read_ptr))
}

fn fifo_storage(
    builder: &mut DigitalBuilder,
    inputs: &FifoInputs,
    write_enable: NetId,
    write_ptr: Vec<NetId>,
    read_ptr: Vec<NetId>,
) -> Result<RegisterFile, SimError> {
    let ports = RegisterPorts {
        write_address: write_ptr,
        write_data: inputs.write_data.to_vec(),
        write_enable,
        read_address_a: read_ptr,
        read_address_b: Vec::new(),
    };
    register_file(builder, inputs.depth, inputs.width, &ports)
}

fn fifo_count_next(
    builder: &mut DigitalBuilder,
    count: &[NetId],
    write_ok: NetId,
    read_ok: NetId,
) -> Result<Vec<NetId>, SimError> {
    let not_read = builder.not(read_ok);
    let write_only = builder.and(write_ok, not_read);
    let not_write = builder.not(write_ok);
    let read_only = builder.and(read_ok, not_write);
    let update = builder.or(write_only, read_only);
    let incremented = increment(builder, count);
    let decremented = decrement(builder, count);
    let stepped = select_vector(builder, &decremented, &incremented, write_ok)?;
    select_vector(builder, count, &stepped, update)
}

fn validate_fifo(width: usize, depth: usize, write_data: &[NetId]) -> Result<(), SimError> {
    if !(2..=32).contains(&depth) || !depth.is_power_of_two() {
        return Err(SimError::InvalidNetlist(
            "FIFO depth must be a power of two in 2..=32".to_string(),
        ));
    }
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "FIFO width must be 1..=64".to_string(),
        ));
    }
    if write_data.len() != width {
        return Err(SimError::InvalidNetlist(
            "FIFO write data width mismatch".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Fifo, fifo_sync};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    const WIDTH: usize = 8;

    fn fifo_fixture(depth: usize) -> (DigitalCircuit, Fifo) {
        let mut builder = DigitalBuilder::new();
        let data = named_inputs(&mut builder, "d", WIDTH);
        let write_enable = builder.input("we");
        let read_enable = builder.input("re");
        let fifo =
            fifo_sync(&mut builder, WIDTH, depth, &data, write_enable, read_enable).expect("fifo");
        (builder.build(), fifo)
    }

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

    fn tick_fifo(circuit: &mut DigitalCircuit, write: bool, read: bool) {
        circuit.set_input("we", write).expect("we");
        circuit.set_input("re", read).expect("re");
        circuit.settle().expect("settle");
        circuit.tick().expect("tick");
    }

    fn fill_fifo(circuit: &mut DigitalCircuit, values: &[u64]) {
        for &value in values {
            set_bits(circuit, "d", value, WIDTH);
            tick_fifo(circuit, true, false);
        }
        circuit.settle().expect("settle");
    }

    fn assert_status(circuit: &DigitalCircuit, fifo: &Fifo, count: u64, full: bool, empty: bool) {
        assert_eq!(read_bits(circuit, &fifo.count), count);
        assert_eq!(circuit.value(fifo.full), full);
        assert_eq!(circuit.value(fifo.empty), empty);
    }

    #[test]
    fn synchronous_fifo_fills_and_drains() {
        let (mut circuit, fifo) = fifo_fixture(4);
        let values = [0x11, 0x22, 0x33, 0x44];
        fill_fifo(&mut circuit, &values);
        assert_status(&circuit, &fifo, 4, true, false);
        assert_eq!(read_bits(&circuit, &fifo.read_data), 0x11);
        tick_fifo(&mut circuit, true, false);
        assert_eq!(read_bits(&circuit, &fifo.count), 4);
        for value in values {
            circuit.settle().expect("settle");
            assert_eq!(read_bits(&circuit, &fifo.read_data), value);
            tick_fifo(&mut circuit, false, true);
        }
        circuit.settle().expect("settle");
        assert_status(&circuit, &fifo, 0, false, true);
    }

    #[test]
    fn synchronous_fifo_reads_and_writes_together() {
        let (mut circuit, fifo) = fifo_fixture(4);
        fill_fifo(&mut circuit, &[0x11, 0x22]);
        set_bits(&mut circuit, "d", 0x33, WIDTH);
        tick_fifo(&mut circuit, true, true);
        circuit.settle().expect("settle");
        assert_status(&circuit, &fifo, 2, false, false);
        assert_eq!(read_bits(&circuit, &fifo.read_data), 0x22);
    }

    #[test]
    fn synchronous_fifo_ignores_read_when_empty() {
        let (mut circuit, fifo) = fifo_fixture(4);
        tick_fifo(&mut circuit, false, true);
        circuit.settle().expect("settle");
        assert_status(&circuit, &fifo, 0, false, true);
    }

    #[test]
    fn fifo_rejects_invalid_shapes() {
        let mut builder = DigitalBuilder::new();
        let data = named_inputs(&mut builder, "d", WIDTH);
        let write_enable = builder.input("we");
        let read_enable = builder.input("re");
        assert!(fifo_sync(&mut builder, WIDTH, 3, &data, write_enable, read_enable).is_err());
        assert!(fifo_sync(&mut builder, WIDTH, 1, &data, write_enable, read_enable).is_err());
        let short = named_inputs(&mut builder, "s", WIDTH / 2);
        assert!(fifo_sync(&mut builder, WIDTH, 4, &short, write_enable, read_enable).is_err());
    }
}
