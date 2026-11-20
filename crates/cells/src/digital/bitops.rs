//! Bit reversal, byte swap, popcount, and saturating arithmetic.

use super::blocks::{
    add_vectors, increment, invert_vector, select_vector, validate_pair, validate_power_of_two,
    validate_width,
};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Reverses the bit order of a vector.
pub fn bit_reverse(bits: &[NetId]) -> Result<Vec<NetId>, SimError> {
    validate_width(bits, "bit reverse", 64)?;
    Ok(bits.iter().rev().copied().collect())
}

/// Swaps bytes within a vector whose width is a multiple of eight.
pub fn byte_swap(bits: &[NetId]) -> Result<Vec<NetId>, SimError> {
    validate_width(bits, "byte swap", 64)?;
    if bits.len() % 8 != 0 {
        return Err(SimError::InvalidNetlist(
            "byte swap width must be a multiple of 8".to_string(),
        ));
    }
    let mut swapped = Vec::with_capacity(bits.len());
    for byte in (0..bits.len() / 8).rev() {
        swapped.extend_from_slice(&bits[byte * 8..byte * 8 + 8]);
    }
    Ok(swapped)
}

/// Counts set bits. Width must be a power of two in 1..=64.
pub fn popcount(builder: &mut DigitalBuilder, data: &[NetId]) -> Result<Vec<NetId>, SimError> {
    validate_power_of_two(data, "popcount", 64)?;
    let count_bits = data.len().trailing_zeros() as usize + 1;
    let zero = builder.constant(false);
    let mut count = vec![zero; count_bits];
    for &bit in data {
        let incremented = increment(builder, &count);
        count = select_vector(builder, &count, &incremented, bit)?;
    }
    Ok(count)
}

/// Adds two unsigned vectors and saturates at the maximum value.
pub fn saturating_add_unsigned(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_pair(first, second, "saturating add", 64)?;
    let zero = builder.constant(false);
    let (sum, carry) = add_vectors(builder, first, second, zero)?;
    let ones = vec![builder.constant(true); first.len()];
    select_vector(builder, &sum, &ones, carry)
}

/// Subtracts two unsigned vectors and saturates at zero.
pub fn saturating_sub_unsigned(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    validate_pair(first, second, "saturating sub", 64)?;
    let one = builder.constant(true);
    let inverted = invert_vector(builder, second);
    let (difference, carry) = add_vectors(builder, first, &inverted, one)?;
    let borrow = builder.not(carry);
    let zeros = vec![builder.constant(false); first.len()];
    select_vector(builder, &difference, &zeros, borrow)
}

#[cfg(test)]
mod tests {
    use super::{
        bit_reverse, byte_swap, popcount, saturating_add_unsigned, saturating_sub_unsigned,
    };
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;
    use kernel::SimError;

    type UnaryBuilder = fn(&mut DigitalBuilder, &[NetId]) -> Result<Vec<NetId>, SimError>;
    type BinaryBuilder =
        fn(&mut DigitalBuilder, &[NetId], &[NetId]) -> Result<Vec<NetId>, SimError>;

    fn unary_fixture(build: UnaryBuilder, width: usize) -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", width);
        let output = build(&mut builder, &data).expect("unary block");
        (builder.build(), output)
    }

    fn binary_fixture(build: BinaryBuilder, width: usize) -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", width);
        let second = inputs(&mut builder, "b", width);
        let output = build(&mut builder, &first, &second).expect("binary block");
        (builder.build(), output)
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
    fn bit_reverse_matches_model() {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", 8);
        let reversed = bit_reverse(&data).expect("reverse");
        let mut circuit = builder.build();
        for value in 0..=255 {
            set_bits(&mut circuit, "d", value, 8);
            circuit.settle().expect("settle");
            let expected = (value as u8).reverse_bits() as u64;
            assert_eq!(read_bits(&circuit, &reversed), expected);
        }
    }

    #[test]
    fn byte_swap_matches_model() {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", 32);
        let swapped = byte_swap(&data).expect("swap");
        let mut circuit = builder.build();
        for value in [0, 1, 0x1234_5678, 0xDEAD_BEEF, 0xFFFF_FFFF] {
            set_bits(&mut circuit, "d", value, 32);
            circuit.settle().expect("settle");
            let expected = (value as u32).swap_bytes() as u64;
            assert_eq!(read_bits(&circuit, &swapped), expected);
        }
    }

    #[test]
    fn popcount_matches_model() {
        let (mut circuit, count) = unary_fixture(popcount, 8);
        for value in 0..=255 {
            set_bits(&mut circuit, "d", value, 8);
            circuit.settle().expect("settle");
            assert_eq!(read_bits(&circuit, &count), value.count_ones() as u64);
        }
    }

    #[test]
    fn saturating_add_matches_model() {
        let (mut circuit, sum) = binary_fixture(saturating_add_unsigned, 8);
        let values = [0, 1, 2, 3, 127, 128, 200, 254, 255];
        for &first in &values {
            for &second in &values {
                set_bits(&mut circuit, "a", first, 8);
                set_bits(&mut circuit, "b", second, 8);
                circuit.settle().expect("settle");
                assert_eq!(read_bits(&circuit, &sum), (first + second).min(255));
            }
        }
    }

    #[test]
    fn saturating_sub_matches_model() {
        let (mut circuit, difference) = binary_fixture(saturating_sub_unsigned, 8);
        let values = [0, 1, 2, 3, 127, 128, 200, 254, 255];
        for &first in &values {
            for &second in &values {
                set_bits(&mut circuit, "a", first, 8);
                set_bits(&mut circuit, "b", second, 8);
                circuit.settle().expect("settle");
                let expected = first.saturating_sub(second);
                assert_eq!(read_bits(&circuit, &difference), expected);
            }
        }
    }

    #[test]
    fn rejects_invalid_widths() {
        assert!(bit_reverse(&[]).is_err());
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", 12);
        assert!(byte_swap(&data).is_err());
        let data = inputs(&mut builder, "e", 6);
        assert!(popcount(&mut builder, &data).is_err());
        let first = inputs(&mut builder, "f", 8);
        let second = inputs(&mut builder, "g", 4);
        assert!(saturating_add_unsigned(&mut builder, &first, &second).is_err());
    }
}
