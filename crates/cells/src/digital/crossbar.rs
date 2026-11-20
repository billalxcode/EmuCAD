//! Parameterized bus multiplexers and crossbars.

use super::blocks::mux_tree;
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Selects one bus from a power-of-two set of equally sized buses.
pub fn bus_mux(
    builder: &mut DigitalBuilder,
    buses: &[&[NetId]],
    select: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_buses(buses)?;
    validate_select(buses.len(), select)?;
    let width = buses[0].len();
    (0..width)
        .map(|bit| {
            let column: Vec<NetId> = buses.iter().map(|bus| bus[bit]).collect();
            mux_tree(builder, &column, select)
        })
        .collect()
}

/// Routes one bus per output from a set of input buses.
pub fn crossbar(
    builder: &mut DigitalBuilder,
    inputs: &[&[NetId]],
    selects: &[&[NetId]],
) -> Result<Vec<Vec<NetId>>, SimError> {
    validate_buses(inputs)?;
    if selects.is_empty() {
        return Err(SimError::InvalidNetlist(
            "crossbar needs at least one output".to_string(),
        ));
    }
    selects
        .iter()
        .map(|select| bus_mux(builder, inputs, select))
        .collect()
}

fn validate_buses(buses: &[&[NetId]]) -> Result<(), SimError> {
    let count = buses.len();
    if !count.is_power_of_two() || count > 64 {
        return Err(SimError::InvalidNetlist(
            "bus count must be a power of two in 1..=64".to_string(),
        ));
    }
    let width = buses[0].len();
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "bus width must be 1..=64".to_string(),
        ));
    }
    if buses.iter().any(|bus| bus.len() != width) {
        return Err(SimError::InvalidNetlist(
            "bus widths must match".to_string(),
        ));
    }
    Ok(())
}

fn validate_select(count: usize, select: &[NetId]) -> Result<(), SimError> {
    if select.len() != count.trailing_zeros() as usize {
        return Err(SimError::InvalidNetlist(
            "bus select width mismatch".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{bus_mux, crossbar};
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

    fn mux_fixture() -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let a = named_inputs(&mut builder, "a", 8);
        let b = named_inputs(&mut builder, "b", 8);
        let c = named_inputs(&mut builder, "c", 8);
        let d = named_inputs(&mut builder, "d", 8);
        let buses: Vec<&[NetId]> = vec![&a, &b, &c, &d];
        let select = named_inputs(&mut builder, "s", 2);
        let output = bus_mux(&mut builder, &buses, &select).expect("bus mux");
        (builder.build(), output)
    }

    #[test]
    fn bus_mux_selects_each_input() {
        let (mut circuit, output) = mux_fixture();
        let values = [0x12, 0x34, 0x56, 0x78];
        for (index, value) in values.iter().enumerate() {
            set_bits(&mut circuit, "a", values[0], 8);
            set_bits(&mut circuit, "b", values[1], 8);
            set_bits(&mut circuit, "c", values[2], 8);
            set_bits(&mut circuit, "d", values[3], 8);
            set_bits(&mut circuit, "s", index as u64, 2);
            circuit.settle().expect("settle");
            assert_eq!(read_bits(&circuit, &output), *value);
        }
    }

    fn crossbar_fixture() -> (DigitalCircuit, Vec<Vec<NetId>>) {
        let mut builder = DigitalBuilder::new();
        let a = named_inputs(&mut builder, "a", 8);
        let b = named_inputs(&mut builder, "b", 8);
        let inputs: Vec<&[NetId]> = vec![&a, &b];
        let select_a = named_inputs(&mut builder, "x", 1);
        let select_b = named_inputs(&mut builder, "y", 1);
        let selects: Vec<&[NetId]> = vec![&select_a, &select_b];
        let outputs = crossbar(&mut builder, &inputs, &selects).expect("crossbar");
        (builder.build(), outputs)
    }

    fn route(circuit: &mut DigitalCircuit, x: u64, y: u64, outputs: &[Vec<NetId>]) -> (u64, u64) {
        set_bits(circuit, "x", x, 1);
        set_bits(circuit, "y", y, 1);
        circuit.settle().expect("settle");
        (
            read_bits(circuit, &outputs[0]),
            read_bits(circuit, &outputs[1]),
        )
    }

    #[test]
    fn crossbar_routes_outputs_independently() {
        let (mut circuit, outputs) = crossbar_fixture();
        set_bits(&mut circuit, "a", 0xAA, 8);
        set_bits(&mut circuit, "b", 0x55, 8);
        assert_eq!(route(&mut circuit, 0, 1, &outputs), (0xAA, 0x55));
        assert_eq!(route(&mut circuit, 1, 0, &outputs), (0x55, 0xAA));
    }

    #[test]
    fn rejects_bad_shapes() {
        let mut builder = DigitalBuilder::new();
        let a = named_inputs(&mut builder, "a", 8);
        let b = named_inputs(&mut builder, "b", 4);
        let buses: Vec<&[NetId]> = vec![&a, &b];
        let select = named_inputs(&mut builder, "s", 1);
        assert!(bus_mux(&mut builder, &buses, &select).is_err());
        let c = named_inputs(&mut builder, "c", 8);
        let buses: Vec<&[NetId]> = vec![&a, &c];
        let bad_select = named_inputs(&mut builder, "t", 2);
        assert!(bus_mux(&mut builder, &buses, &bad_select).is_err());
    }
}
