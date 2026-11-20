//! Combinational barrel, rotate, and funnel shifters.

use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Builds a logical left barrel shifter.
pub fn barrel_shift_left(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_shift(data, amount)?;
    let zero = zero_vector(builder, data.len());
    Ok(funnel_left_wide(builder, data, &zero, amount))
}

/// Builds a logical right barrel shifter.
pub fn barrel_shift_right(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_shift(data, amount)?;
    let zero = zero_vector(builder, data.len());
    Ok(funnel_right_wide(builder, data, &zero, amount))
}

/// Builds an arithmetic right barrel shifter that replicates the sign bit.
pub fn arithmetic_shift_right(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_shift(data, amount)?;
    let sign = data[data.len() - 1];
    let fill = vec![sign; data.len()];
    Ok(funnel_right_wide(builder, data, &fill, amount))
}

/// Builds a left rotate.
pub fn rotate_left(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_shift(data, amount)?;
    Ok(funnel_left_wide(builder, data, data, amount))
}

/// Builds a right rotate.
pub fn rotate_right(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_shift(data, amount)?;
    Ok(funnel_right_wide(builder, data, data, amount))
}

/// Builds a funnel shift left of `{high, low}`.
pub fn funnel_shift_left(
    builder: &mut DigitalBuilder,
    high: &[NetId],
    low: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_pair(high, low, amount)?;
    Ok(funnel_left_wide(builder, high, low, amount))
}

/// Builds a funnel shift right of `{high, low}`.
pub fn funnel_shift_right(
    builder: &mut DigitalBuilder,
    high: &[NetId],
    low: &[NetId],
    amount: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_pair(high, low, amount)?;
    Ok(funnel_right_wide(builder, high, low, amount))
}

fn funnel_left_wide(
    builder: &mut DigitalBuilder,
    high: &[NetId],
    low: &[NetId],
    amount: &[NetId],
) -> Vec<NetId> {
    let width = high.len();
    let zero = builder.constant(false);
    let mut wide = Vec::with_capacity(width * 2);
    wide.extend_from_slice(low);
    wide.extend_from_slice(high);
    for (stage, &select) in amount.iter().enumerate() {
        wide = shift_wide_left(builder, &wide, 1 << stage, select, zero);
    }
    wide[width..].to_vec()
}

fn funnel_right_wide(
    builder: &mut DigitalBuilder,
    high: &[NetId],
    low: &[NetId],
    amount: &[NetId],
) -> Vec<NetId> {
    let reversed_high: Vec<NetId> = high.iter().rev().copied().collect();
    let reversed_low: Vec<NetId> = low.iter().rev().copied().collect();
    let output = funnel_left_wide(builder, &reversed_high, &reversed_low, amount);
    output.into_iter().rev().collect()
}

fn shift_wide_left(
    builder: &mut DigitalBuilder,
    wide: &[NetId],
    distance: usize,
    select: NetId,
    zero: NetId,
) -> Vec<NetId> {
    let len = wide.len();
    (0..len)
        .map(|index| {
            if index < distance {
                zero
            } else {
                builder.mux(wide[index], wide[index - distance], select)
            }
        })
        .collect()
}

fn zero_vector(builder: &mut DigitalBuilder, width: usize) -> Vec<NetId> {
    let zero = builder.constant(false);
    vec![zero; width]
}

fn validate_shift(data: &[NetId], amount: &[NetId]) -> Result<(), SimError> {
    let width = data.len();
    if !width.is_power_of_two() || width > 64 {
        return Err(invalid("shift width must be a power of two in 1..=64"));
    }
    if amount.len() != width.trailing_zeros() as usize {
        return Err(invalid("shift amount width mismatch"));
    }
    Ok(())
}

fn validate_pair(high: &[NetId], low: &[NetId], amount: &[NetId]) -> Result<(), SimError> {
    if high.len() != low.len() {
        return Err(invalid("funnel shift operands must have the same width"));
    }
    validate_shift(high, amount)
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        arithmetic_shift_right, barrel_shift_left, barrel_shift_right, funnel_shift_left,
        funnel_shift_right, rotate_left, rotate_right,
    };
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use kernel::SimError;

    const WIDTH: usize = 32;
    const AMOUNT_WIDTH: usize = 5;
    const MASK: u64 = 0xFFFF_FFFF;

    type ShiftBuilder =
        fn(&mut DigitalBuilder, &[NetId], &[NetId], &[NetId]) -> Result<Vec<NetId>, SimError>;

    fn fixture(build: ShiftBuilder) -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "d", WIDTH);
        let second = inputs(&mut builder, "e", WIDTH);
        let amount = inputs(&mut builder, "s", AMOUNT_WIDTH);
        let result = build(&mut builder, &first, &second, &amount).expect("shift");
        for (index, &net) in result.iter().enumerate() {
            builder.output(&format!("y{index}"), net);
        }
        builder.build()
    }

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn evaluate(circuit: &mut DigitalCircuit, first: u64, second: u64, amount: u64) -> u64 {
        set_bits(circuit, "d", first, WIDTH);
        set_bits(circuit, "e", second, WIDTH);
        set_bits(circuit, "s", amount, AMOUNT_WIDTH);
        circuit.settle().expect("settle");
        let mut value = 0;
        for index in 0..WIDTH {
            if circuit.output(&format!("y{index}")).expect("output") {
                value |= 1 << index;
            }
        }
        value
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn model_funnel_left(high: u64, low: u64, shift: u32) -> u64 {
        let wide = ((high as u128) << WIDTH) | low as u128;
        ((wide << shift) >> WIDTH) as u64 & MASK
    }

    fn model_funnel_right(high: u64, low: u64, shift: u32) -> u64 {
        let wide = ((low as u128) << WIDTH) | high as u128;
        (wide >> shift) as u64 & MASK
    }

    #[test]
    fn logical_left_shift_matches_model() {
        let mut circuit =
            fixture(|builder, first, _, amount| barrel_shift_left(builder, first, amount));
        for shift in 0..WIDTH as u64 {
            let value = 0x8000_0001;
            assert_eq!(
                evaluate(&mut circuit, value, 0, shift),
                (value << shift) & MASK
            );
        }
    }

    #[test]
    fn logical_right_shift_matches_model() {
        let mut circuit =
            fixture(|builder, first, _, amount| barrel_shift_right(builder, first, amount));
        for shift in 0..WIDTH as u64 {
            let value = 0x8000_0001;
            assert_eq!(evaluate(&mut circuit, value, 0, shift), value >> shift);
        }
    }

    #[test]
    fn arithmetic_right_shift_replicates_sign() {
        let mut circuit =
            fixture(|builder, first, _, amount| arithmetic_shift_right(builder, first, amount));
        for shift in 0..WIDTH as u32 {
            for value in [0x8000_0000, 0x4000_0000, 0xFFFF_FFFF] {
                let expected = ((value as u32 as i32) >> shift) as u32 as u64;
                assert_eq!(evaluate(&mut circuit, value, 0, shift as u64), expected);
            }
        }
    }

    #[test]
    fn rotate_matches_model() {
        let mut left = fixture(|builder, first, _, amount| rotate_left(builder, first, amount));
        let mut right = fixture(|builder, first, _, amount| rotate_right(builder, first, amount));
        let value = 0x1234_5678;
        for shift in 0..WIDTH as u32 {
            let expected_left = (value as u32).rotate_left(shift) as u64;
            let expected_right = (value as u32).rotate_right(shift) as u64;
            assert_eq!(evaluate(&mut left, value, 0, shift as u64), expected_left);
            assert_eq!(evaluate(&mut right, value, 0, shift as u64), expected_right);
        }
    }

    #[test]
    fn funnel_shifts_match_model() {
        let mut left = fixture(funnel_shift_left);
        let mut right = fixture(funnel_shift_right);
        let high = 0x1122_3344;
        let low = 0x5566_7788;
        for shift in 0..WIDTH as u32 {
            let expected_left = model_funnel_left(high, low, shift);
            let expected_right = model_funnel_right(high, low, shift);
            assert_eq!(evaluate(&mut left, high, low, shift as u64), expected_left);
            assert_eq!(
                evaluate(&mut right, high, low, shift as u64),
                expected_right
            );
        }
    }

    #[test]
    fn rejects_non_power_of_two_width() {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", 24);
        let amount = inputs(&mut builder, "s", 5);
        assert!(barrel_shift_left(&mut builder, &data, &amount).is_err());
    }

    #[test]
    fn rejects_mismatched_funnel_operands() {
        let mut builder = DigitalBuilder::new();
        let high = inputs(&mut builder, "d", WIDTH);
        let low = inputs(&mut builder, "e", 16);
        let amount = inputs(&mut builder, "s", AMOUNT_WIDTH);
        assert!(funnel_shift_left(&mut builder, &high, &low, &amount).is_err());
    }

    #[test]
    fn rejects_wrong_amount_width() {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", WIDTH);
        let amount = inputs(&mut builder, "s", AMOUNT_WIDTH - 1);
        assert!(barrel_shift_right(&mut builder, &data, &amount).is_err());
    }
}
