//! Combinational unsigned magnitude comparator.

use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Result of comparing two unsigned vectors.
pub struct Comparison {
    /// First operand is less than the second.
    pub less: NetId,
    /// Both operands are equal.
    pub equal: NetId,
    /// First operand is greater than the second.
    pub greater: NetId,
}

/// Builds an unsigned comparator for two equally sized vectors.
pub fn comparator_unsigned(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
) -> Result<Comparison, SimError> {
    validate_comparator(first, second)?;
    let mut state = CompareState {
        less: builder.constant(false),
        equal: builder.constant(true),
        greater: builder.constant(false),
    };
    for index in (0..first.len()).rev() {
        state = compare_bit(builder, first[index], second[index], state);
    }
    Ok(Comparison {
        less: state.less,
        equal: state.equal,
        greater: state.greater,
    })
}

struct CompareState {
    less: NetId,
    equal: NetId,
    greater: NetId,
}

fn compare_bit(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    state: CompareState,
) -> CompareState {
    let not_first = builder.not(first);
    let not_second = builder.not(second);
    let bit_less = builder.and(not_first, second);
    let bit_greater = builder.and(first, not_second);
    let difference = builder.xor(first, second);
    let bit_equal = builder.not(difference);
    let lower = builder.and(state.equal, bit_less);
    let higher = builder.and(state.equal, bit_greater);
    CompareState {
        less: builder.or(state.less, lower),
        equal: builder.and(state.equal, bit_equal),
        greater: builder.or(state.greater, higher),
    }
}

fn validate_comparator(first: &[NetId], second: &[NetId]) -> Result<(), SimError> {
    if first.len() != second.len() {
        return Err(invalid("comparator operand width mismatch"));
    }
    if first.is_empty() || first.len() > 64 {
        return Err(invalid("comparator width must be 1..=64"));
    }
    Ok(())
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Comparison, comparator_unsigned};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};

    const WIDTH: usize = 8;

    fn fixture() -> (DigitalCircuit, Comparison) {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", WIDTH);
        let second = inputs(&mut builder, "b", WIDTH);
        let comparison = comparator_unsigned(&mut builder, &first, &second).expect("comparator");
        (builder.build(), comparison)
    }

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64) {
        for index in 0..WIDTH {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        comparison: &Comparison,
        first: u64,
        second: u64,
    ) -> (bool, bool, bool) {
        set_bits(circuit, "a", first);
        set_bits(circuit, "b", second);
        circuit.settle().expect("settle");
        (
            circuit.value(comparison.less),
            circuit.value(comparison.equal),
            circuit.value(comparison.greater),
        )
    }

    #[test]
    fn comparator_matches_unsigned_model() {
        let (mut circuit, comparison) = fixture();
        let values = [0, 1, 2, 7, 8, 127, 128, 254, 255];
        for &first in &values {
            for &second in &values {
                let actual = evaluate(&mut circuit, &comparison, first, second);
                let expected = (first < second, first == second, first > second);
                assert_eq!(actual, expected, "{first} vs {second}");
            }
        }
    }

    #[test]
    fn rejects_width_mismatch() {
        let mut builder = DigitalBuilder::new();
        let first = inputs(&mut builder, "a", WIDTH);
        let second = inputs(&mut builder, "b", WIDTH / 2);
        assert!(comparator_unsigned(&mut builder, &first, &second).is_err());
    }

    #[test]
    fn rejects_empty_operands() {
        let mut builder = DigitalBuilder::new();
        assert!(comparator_unsigned(&mut builder, &[], &[]).is_err());
    }
}
