//! Shared bit helpers for catalog digital circuits.

use crate::{DigitalBuilder, NetId};

/// Returns true when a four-bit value equals a constant.
pub(crate) fn equals4(builder: &mut DigitalBuilder, bits: &[NetId; 4], value: u8) -> NetId {
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
