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

/// Returns the four-bit increment of a value.
pub(crate) fn increment4(builder: &mut DigitalBuilder, bits: &[NetId; 4]) -> [NetId; 4] {
    let not0 = builder.not(bits[0]);
    let carry0 = bits[0];
    let bit1 = builder.xor(bits[1], carry0);
    let carry1 = builder.and(carry0, bits[1]);
    let bit2 = builder.xor(bits[2], carry1);
    let carry2 = builder.and(carry1, bits[2]);
    let bit3 = builder.xor(bits[3], carry2);
    [not0, bit1, bit2, bit3]
}
