//! Priority and one-hot encoders plus leading-zero count.

use super::blocks::{increment, or_all, select_vector, validate_power_of_two};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Result of a priority encoder.
pub struct PriorityEncoding {
    /// Encoded index, least significant bit first.
    pub index: Vec<NetId>,
    /// At least one input is high.
    pub valid: NetId,
}

/// Encodes the lowest set input index.
///
/// Input count must be a power of two in 1..=64.
pub fn priority_encoder(
    builder: &mut DigitalBuilder,
    inputs: &[NetId],
) -> Result<PriorityEncoding, SimError> {
    validate_power_of_two(inputs, "encoder", 64)?;
    let lines = priority_lines(builder, inputs);
    let index = encode_lines(builder, &lines);
    Ok(PriorityEncoding {
        index,
        valid: or_all(builder, inputs),
    })
}

/// Encodes a one-hot input vector into a binary index.
///
/// Input count must be a power of two in 1..=64.
pub fn one_hot_encoder(
    builder: &mut DigitalBuilder,
    inputs: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_power_of_two(inputs, "encoder", 64)?;
    Ok(encode_lines(builder, inputs))
}

/// Counts leading zeros in a vector.
///
/// Width must be a power of two in 1..=64.
pub fn leading_zero_count(
    builder: &mut DigitalBuilder,
    data: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_power_of_two(data, "encoder", 64)?;
    let width = data.len();
    let count_bits = width.trailing_zeros() as usize + 1;
    let zero = builder.constant(false);
    let mut count = vec![zero; count_bits];
    let mut found = zero;
    for index in (0..width).rev() {
        let not_found = builder.not(found);
        let not_bit = builder.not(data[index]);
        let advance = builder.and(not_found, not_bit);
        let incremented = increment(builder, &count);
        count = select_vector(builder, &count, &incremented, advance)?;
        found = builder.or(found, data[index]);
    }
    Ok(count)
}

fn priority_lines(builder: &mut DigitalBuilder, inputs: &[NetId]) -> Vec<NetId> {
    let mut blocker = builder.constant(false);
    let mut lines = Vec::with_capacity(inputs.len());
    for &input in inputs {
        let free = builder.not(blocker);
        lines.push(builder.and(input, free));
        blocker = builder.or(blocker, input);
    }
    lines
}

fn encode_lines(builder: &mut DigitalBuilder, lines: &[NetId]) -> Vec<NetId> {
    let bits = lines.len().trailing_zeros() as usize;
    (0..bits)
        .map(|bit| {
            let terms: Vec<NetId> = lines
                .iter()
                .enumerate()
                .filter(|(index, _)| index & (1 << bit) != 0)
                .map(|(_, &line)| line)
                .collect();
            or_all(builder, &terms)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PriorityEncoding, leading_zero_count, one_hot_encoder, priority_encoder};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    const WIDTH: usize = 8;

    fn priority_fixture() -> (DigitalCircuit, PriorityEncoding) {
        let mut builder = DigitalBuilder::new();
        let inputs = inputs(&mut builder, "i", WIDTH);
        let encoding = priority_encoder(&mut builder, &inputs).expect("priority");
        (builder.build(), encoding)
    }

    fn one_hot_fixture() -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let inputs = inputs(&mut builder, "i", WIDTH);
        let index = one_hot_encoder(&mut builder, &inputs).expect("one hot");
        (builder.build(), index)
    }

    fn lzc_fixture() -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", WIDTH);
        let count = leading_zero_count(&mut builder, &data).expect("lzc");
        (builder.build(), count)
    }

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
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

    #[test]
    fn priority_encoder_picks_lowest_set_bit() {
        let (mut circuit, encoding) = priority_fixture();
        for pattern in 0..=255u64 {
            set_bits(&mut circuit, "i", pattern, WIDTH);
            circuit.settle().expect("settle");
            let expected = (pattern != 0).then(|| pattern.trailing_zeros() as u64);
            assert_eq!(
                circuit.value(encoding.valid),
                expected.is_some(),
                "pattern {pattern}"
            );
            assert_eq!(read_bits(&circuit, &encoding.index), expected.unwrap_or(0));
        }
    }

    #[test]
    fn one_hot_encoder_maps_each_position() {
        let (mut circuit, index) = one_hot_fixture();
        for position in 0..WIDTH {
            set_bits(&mut circuit, "i", 1 << position, WIDTH);
            circuit.settle().expect("settle");
            assert_eq!(read_bits(&circuit, &index), position as u64);
        }
    }

    #[test]
    fn leading_zero_count_matches_model() {
        let (mut circuit, count) = lzc_fixture();
        for pattern in 0..=255u64 {
            set_bits(&mut circuit, "d", pattern, WIDTH);
            circuit.settle().expect("settle");
            let expected = (pattern as u8).leading_zeros() as u64;
            assert_eq!(read_bits(&circuit, &count), expected, "pattern {pattern}");
        }
    }

    #[test]
    fn rejects_non_power_of_two_width() {
        let mut builder = DigitalBuilder::new();
        let inputs = inputs(&mut builder, "i", 6);
        assert!(priority_encoder(&mut builder, &inputs).is_err());
        assert!(one_hot_encoder(&mut builder, &[]).is_err());
    }
}
