//! Combinational array and Booth multipliers.

use super::blocks::{add_vectors, validate_operands};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Builds an unsigned array multiplier.
///
/// Returns the two-width product, least significant bit first.
pub fn multiplier_array(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_operands(first, second, "multiplier", 32)?;
    let width = first.len();
    let zero = builder.constant(false);
    let mut acc = vec![zero; width * 2];
    for (index, &bit) in second.iter().enumerate() {
        let row = array_row(builder, first, bit, index, zero);
        acc = add_vectors(builder, &acc, &row, zero)?.0;
    }
    Ok(acc)
}

/// Builds a signed Booth multiplier for two's complement operands.
///
/// Returns the two-width product, least significant bit first.
pub fn multiplier_booth(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_operands(first, second, "multiplier", 32)?;
    let width = first.len();
    let zero = builder.constant(false);
    let wide = sign_extend(first);
    let mut acc = vec![zero; width * 2];
    let mut previous = zero;
    for (index, &bit) in second.iter().enumerate() {
        let (active, subtract) = booth_code(builder, bit, previous);
        let row = booth_row(builder, &wide, index, zero, active, subtract);
        acc = add_vectors(builder, &acc, &row, subtract)?.0;
        previous = bit;
    }
    Ok(acc)
}

fn array_row(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    enable: NetId,
    offset: usize,
    zero: NetId,
) -> Vec<NetId> {
    let mut row = vec![zero; first.len() * 2];
    for (index, &bit) in first.iter().enumerate() {
        row[offset + index] = builder.and(bit, enable);
    }
    row
}

fn booth_code(builder: &mut DigitalBuilder, bit: NetId, previous: NetId) -> (NetId, NetId) {
    let active = builder.xor(bit, previous);
    let subtract = builder.and(bit, active);
    (active, subtract)
}

fn booth_row(
    builder: &mut DigitalBuilder,
    wide: &[NetId],
    offset: usize,
    zero: NetId,
    active: NetId,
    subtract: NetId,
) -> Vec<NetId> {
    let shifted = shift_left_nets(wide, offset, zero);
    conditional_row(builder, &shifted, active, subtract)
}

fn shift_left_nets(wide: &[NetId], distance: usize, zero: NetId) -> Vec<NetId> {
    (0..wide.len())
        .map(|index| {
            if index < distance {
                zero
            } else {
                wide[index - distance]
            }
        })
        .collect()
}

fn conditional_row(
    builder: &mut DigitalBuilder,
    shifted: &[NetId],
    active: NetId,
    subtract: NetId,
) -> Vec<NetId> {
    shifted
        .iter()
        .map(|&bit| {
            let masked = builder.xor(bit, subtract);
            builder.and(masked, active)
        })
        .collect()
}

fn sign_extend(data: &[NetId]) -> Vec<NetId> {
    let sign = data[data.len() - 1];
    let mut wide = data.to_vec();
    wide.extend(std::iter::repeat_n(sign, data.len()));
    wide
}

#[cfg(test)]
mod tests {
    use super::{multiplier_array, multiplier_booth};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use kernel::SimError;

    const WIDTH: usize = 8;
    const PRODUCT_WIDTH: usize = WIDTH * 2;

    type MultiplierBuilder =
        fn(&mut DigitalBuilder, &[NetId], &[NetId]) -> Result<Vec<NetId>, SimError>;

    fn fixture(build: MultiplierBuilder) -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", WIDTH);
        let second = inputs(&mut builder, "b", WIDTH);
        let product = build(&mut builder, &first, &second).expect("multiplier");
        for (index, &net) in product.iter().enumerate() {
            builder.output(&format!("y{index}"), net);
        }
        builder.build()
    }

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn evaluate(circuit: &mut DigitalCircuit, first: u64, second: u64) -> u64 {
        set_bits(circuit, "a", first);
        set_bits(circuit, "b", second);
        circuit.settle().expect("settle");
        let mut value = 0;
        for index in 0..PRODUCT_WIDTH {
            if circuit.output(&format!("y{index}")).expect("output") {
                value |= 1 << index;
            }
        }
        value
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64) {
        for index in 0..WIDTH {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    #[test]
    fn array_multiplier_matches_unsigned_model() {
        let mut circuit = fixture(multiplier_array);
        for first in 0..16 {
            for second in 0..16 {
                assert_eq!(evaluate(&mut circuit, first, second), first * second);
            }
        }
        assert_eq!(evaluate(&mut circuit, 0xFF, 0xFF), 0xFE01);
    }

    #[test]
    fn booth_multiplier_matches_signed_model() {
        let mut circuit = fixture(multiplier_booth);
        let values: [i32; 8] = [0, 1, -1, 2, -2, 7, -8, -128];
        for &first in &values {
            for &second in &values {
                let expected = (first * second) as u16 as u64;
                let first = first as u8 as u64;
                let second = second as u8 as u64;
                assert_eq!(evaluate(&mut circuit, first, second), expected);
            }
        }
    }

    #[test]
    fn rejects_non_power_of_two_width() {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", 6);
        let second = inputs(&mut builder, "b", 6);
        assert!(multiplier_array(&mut builder, &first, &second).is_err());
    }

    #[test]
    fn rejects_mismatched_operand_width() {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", WIDTH);
        let second = inputs(&mut builder, "b", WIDTH / 2);
        assert!(multiplier_booth(&mut builder, &first, &second).is_err());
    }
}
