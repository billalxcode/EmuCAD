//! Shared bit helpers for catalog digital circuits.

use crate::{DigitalBuilder, NetId};

/// Returns true when a bit vector equals a constant.
pub(crate) fn equals(builder: &mut DigitalBuilder, bits: &[NetId], value: u32) -> NetId {
    let mut result = builder.constant(true);
    for (index, bit) in bits.iter().enumerate() {
        let literal = if value & (1 << index) != 0 {
            *bit
        } else {
            builder.not(*bit)
        };
        result = builder.and(result, literal);
    }
    result
}

/// Selects one net from a power-of-two list using a mux tree.
pub(crate) fn mux_tree(
    builder: &mut DigitalBuilder,
    mut level: Vec<NetId>,
    selects: &[NetId],
) -> NetId {
    for select in selects {
        level = level
            .chunks(2)
            .map(|pair| builder.mux(pair[0], pair[1], *select))
            .collect();
    }
    level[0]
}

/// Shifts a bit vector left and inserts a new least-significant bit.
pub(crate) fn shift_in<const N: usize>(bits: &[NetId; N], input: NetId) -> [NetId; N] {
    std::array::from_fn(|index| if index == 0 { input } else { bits[index - 1] })
}

/// Returns true when a bit vector is less than a constant.
pub(crate) fn less_than(builder: &mut DigitalBuilder, bits: &[NetId], value: u32) -> NetId {
    let mut equal = builder.constant(true);
    let mut less = builder.constant(false);
    for (index, bit) in bits.iter().enumerate().rev() {
        let not_bit = builder.not(*bit);
        if value & (1 << index) != 0 {
            let below = builder.and(equal, not_bit);
            less = builder.or(less, below);
            equal = builder.and(equal, *bit);
        } else {
            equal = builder.and(equal, not_bit);
        }
    }
    less
}

/// Stores values into existing nets when enabled.
pub(crate) fn register_bits(
    builder: &mut DigitalBuilder,
    outputs: &[NetId],
    values: &[NetId],
    enable: NetId,
) {
    for (output, value) in outputs.iter().zip(values) {
        let next = builder.and(*value, enable);
        builder.dff_into(next, *output);
    }
}

/// Returns the increment of a bit vector.
pub(crate) fn increment(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut carry = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for bit in bits {
        output.push(builder.xor(*bit, carry));
        carry = builder.and(carry, *bit);
    }
    output
}
