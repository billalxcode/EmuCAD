//! Combinational restoring and non-restoring dividers.

use super::blocks::{add_vectors, select_vector, validate_operands};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Quotient and remainder of an unsigned division.
pub struct Division {
    /// Quotient, least significant bit first.
    pub quotient: Vec<NetId>,
    /// Remainder, least significant bit first.
    pub remainder: Vec<NetId>,
}

/// Builds an unsigned restoring divider.
///
/// Division by zero is undefined.
pub fn divider_restoring(
    builder: &mut DigitalBuilder,
    dividend: &[NetId],
    divisor: &[NetId],
) -> Result<Division, SimError> {
    validate_operands(dividend, divisor, "divider", 32)?;
    let width = dividend.len();
    let zero = builder.constant(false);
    let one = builder.constant(true);
    let inverted = invert_vector(builder, &zero_extend(divisor, zero));
    let (quotient, remainder) = restore_quotient(builder, dividend, &inverted, width, zero, one)?;
    Ok(Division {
        quotient,
        remainder: remainder[..width].to_vec(),
    })
}

/// Builds an unsigned non-restoring divider.
///
/// Division by zero is undefined.
pub fn divider_non_restoring(
    builder: &mut DigitalBuilder,
    dividend: &[NetId],
    divisor: &[NetId],
) -> Result<Division, SimError> {
    validate_operands(dividend, divisor, "divider", 32)?;
    let width = dividend.len();
    let zero = builder.constant(false);
    let wide_divisor = zero_extend(divisor, zero);
    let (quotient, remainder) =
        non_restore_quotient(builder, dividend, &wide_divisor, width, zero)?;
    let negative = remainder[width];
    let addend = and_vector(builder, &wide_divisor, negative);
    let (corrected, _) = add_vectors(builder, &remainder, &addend, zero)?;
    Ok(Division {
        quotient,
        remainder: corrected[..width].to_vec(),
    })
}

fn restore_quotient(
    builder: &mut DigitalBuilder,
    dividend: &[NetId],
    inverted: &[NetId],
    width: usize,
    zero: NetId,
    one: NetId,
) -> Result<(Vec<NetId>, Vec<NetId>), SimError> {
    let mut remainder = vec![zero; width + 1];
    let mut quotient = vec![zero; width];
    for bit in (0..width).rev() {
        let shifted = shift_in(&remainder, dividend[bit]);
        let (difference, carry) = add_vectors(builder, &shifted, inverted, one)?;
        remainder = select_vector(builder, &shifted, &difference, carry)?;
        quotient[bit] = carry;
    }
    Ok((quotient, remainder))
}

fn non_restore_quotient(
    builder: &mut DigitalBuilder,
    dividend: &[NetId],
    wide_divisor: &[NetId],
    width: usize,
    zero: NetId,
) -> Result<(Vec<NetId>, Vec<NetId>), SimError> {
    let mut remainder = vec![zero; width + 1];
    let mut quotient = vec![zero; width];
    for bit in (0..width).rev() {
        let negative = remainder[width];
        let shifted = shift_in(&remainder, dividend[bit]);
        let subtract = builder.not(negative);
        let addend = xor_vector(builder, wide_divisor, subtract);
        let (sum, _) = add_vectors(builder, &shifted, &addend, subtract)?;
        quotient[bit] = builder.not(sum[width]);
        remainder = sum;
    }
    Ok((quotient, remainder))
}

fn shift_in(remainder: &[NetId], bit: NetId) -> Vec<NetId> {
    let mut next = Vec::with_capacity(remainder.len());
    next.push(bit);
    next.extend_from_slice(&remainder[..remainder.len() - 1]);
    next
}

fn invert_vector(builder: &mut DigitalBuilder, data: &[NetId]) -> Vec<NetId> {
    data.iter().map(|&bit| builder.not(bit)).collect()
}

fn xor_vector(builder: &mut DigitalBuilder, data: &[NetId], control: NetId) -> Vec<NetId> {
    data.iter().map(|&bit| builder.xor(bit, control)).collect()
}

fn and_vector(builder: &mut DigitalBuilder, data: &[NetId], control: NetId) -> Vec<NetId> {
    data.iter().map(|&bit| builder.and(bit, control)).collect()
}

fn zero_extend(data: &[NetId], zero: NetId) -> Vec<NetId> {
    let mut wide = data.to_vec();
    wide.push(zero);
    wide
}

#[cfg(test)]
mod tests {
    use super::{Division, divider_non_restoring, divider_restoring};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use kernel::SimError;

    const WIDTH: usize = 8;

    type DividerBuilder = fn(&mut DigitalBuilder, &[NetId], &[NetId]) -> Result<Division, SimError>;

    fn fixture(build: DividerBuilder) -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let dividend = inputs(&mut builder, "a", WIDTH);
        let divisor = inputs(&mut builder, "b", WIDTH);
        let division = build(&mut builder, &dividend, &divisor).expect("divider");
        expose(&mut builder, "q", &division.quotient);
        expose(&mut builder, "r", &division.remainder);
        builder.build()
    }

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn expose(builder: &mut DigitalBuilder, prefix: &str, nets: &[NetId]) {
        for (index, &net) in nets.iter().enumerate() {
            builder.output(&format!("{prefix}{index}"), net);
        }
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64) {
        for index in 0..WIDTH {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn evaluate(circuit: &mut DigitalCircuit, dividend: u64, divisor: u64) -> (u64, u64) {
        set_bits(circuit, "a", dividend);
        set_bits(circuit, "b", divisor);
        circuit.settle().expect("settle");
        (read_output(circuit, "q"), read_output(circuit, "r"))
    }

    fn read_output(circuit: &DigitalCircuit, prefix: &str) -> u64 {
        let mut value = 0;
        for index in 0..WIDTH {
            if circuit.output(&format!("{prefix}{index}")).expect("output") {
                value |= 1 << index;
            }
        }
        value
    }

    fn check_division(circuit: &mut DigitalCircuit, dividend: u64, divisor: u64) {
        let (quotient, remainder) = evaluate(circuit, dividend, divisor);
        assert_eq!(
            quotient,
            dividend / divisor,
            "quotient for {dividend}/{divisor}"
        );
        assert_eq!(
            remainder,
            dividend % divisor,
            "remainder for {dividend}/{divisor}"
        );
    }

    #[test]
    fn restoring_division_matches_model() {
        let mut circuit = fixture(divider_restoring);
        let divisors = [
            1, 2, 3, 4, 5, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 129, 200, 254, 255,
        ];
        for divisor in divisors {
            for dividend in 0..=255 {
                check_division(&mut circuit, dividend, divisor);
            }
        }
    }

    #[test]
    fn non_restoring_division_matches_model() {
        let mut circuit = fixture(divider_non_restoring);
        let divisors = [
            1, 2, 3, 4, 5, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 129, 200, 254, 255,
        ];
        for divisor in divisors {
            for dividend in 0..=255 {
                check_division(&mut circuit, dividend, divisor);
            }
        }
    }

    #[test]
    fn rejects_mismatched_operand_width() {
        let mut builder = DigitalBuilder::new();
        let dividend = inputs(&mut builder, "a", WIDTH);
        let divisor = inputs(&mut builder, "b", WIDTH / 2);
        assert!(divider_restoring(&mut builder, &dividend, &divisor).is_err());
    }

    #[test]
    fn rejects_non_power_of_two_width() {
        let mut builder = DigitalBuilder::new();
        let dividend = inputs(&mut builder, "a", 6);
        let divisor = inputs(&mut builder, "b", 6);
        assert!(divider_non_restoring(&mut builder, &dividend, &divisor).is_err());
    }
}
