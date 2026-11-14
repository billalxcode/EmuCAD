use super::*;

/// Flags produced by an AVR compare operation.
pub struct CompareUnit {
    /// Carry or borrow.
    pub carry: NetId,
    /// Result is zero.
    pub zero: NetId,
    /// Result bit 7.
    pub negative: NetId,
    /// Signed overflow.
    pub overflow: NetId,
    /// Signed result sign.
    pub sign: NetId,
    /// Carry or borrow from bit 3.
    pub half_carry: NetId,
}

/// Builds a compare unit. The arithmetic result is not written back.
pub fn compare_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    carry_in: NetId,
) -> Result<CompareUnit, SimError> {
    let subtract = builder.constant(true);
    let unit = arithmetic_unit(builder, first, second, subtract, carry_in)?;
    Ok(CompareUnit {
        carry: unit.carry,
        zero: unit.zero,
        negative: unit.negative,
        overflow: unit.overflow,
        sign: unit.sign,
        half_carry: unit.half_carry,
    })
}

/// Builds a NEG unit: result is zero minus the input.
pub fn negate_unit(
    builder: &mut DigitalBuilder,
    input: &[NetId],
) -> Result<ArithmeticUnit, SimError> {
    let zero = constant_byte(builder, 0);
    let subtract = builder.constant(true);
    let carry = builder.constant(false);
    arithmetic_unit(builder, &zero, input, subtract, carry)
}

/// Builds an INC unit: result is the input plus one.
pub fn increment_unit(
    builder: &mut DigitalBuilder,
    input: &[NetId],
) -> Result<ArithmeticUnit, SimError> {
    let one = constant_byte(builder, 1);
    let add = builder.constant(false);
    let carry = builder.constant(false);
    arithmetic_unit(builder, input, &one, add, carry)
}

/// Builds a DEC unit: result is the input minus one.
pub fn decrement_unit(
    builder: &mut DigitalBuilder,
    input: &[NetId],
) -> Result<ArithmeticUnit, SimError> {
    let one = constant_byte(builder, 1);
    let subtract = builder.constant(true);
    let carry = builder.constant(false);
    arithmetic_unit(builder, input, &one, subtract, carry)
}

fn constant_byte(builder: &mut DigitalBuilder, value: u8) -> Vec<NetId> {
    let zero = builder.constant(false);
    let one = builder.constant(true);
    (0..8)
        .map(|bit| if value & (1 << bit) != 0 { one } else { zero })
        .collect()
}
