use super::*;

pub struct ArithmeticUnit {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
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

struct SumBits {
    sum: Vec<NetId>,
    carry: NetId,
    half_carry: NetId,
}

/// Builds the 8-bit AVR arithmetic unit.
///
/// `subtract` selects subtraction. `carry_in` is the C flag for ADC or SBC.
pub fn arithmetic_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    carry_in: NetId,
) -> Result<ArithmeticUnit, SimError> {
    validate(first, second)?;
    let bits = add_bytes(builder, first, second, subtract, carry_in);
    Ok(finish_unit(builder, first, second, bits, subtract))
}

fn finish_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    bits: SumBits,
    subtract: NetId,
) -> ArithmeticUnit {
    let flags = unit_flags(builder, first, second, &bits, subtract);
    ArithmeticUnit {
        result: bits.sum,
        carry: flags.carry,
        zero: flags.zero,
        negative: flags.negative,
        overflow: flags.overflow,
        sign: flags.sign,
        half_carry: flags.half_carry,
    }
}

struct UnitFlags {
    carry: NetId,
    zero: NetId,
    negative: NetId,
    overflow: NetId,
    sign: NetId,
    half_carry: NetId,
}

fn unit_flags(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    bits: &SumBits,
    subtract: NetId,
) -> UnitFlags {
    let overflow = overflow_flag(builder, first, second, &bits.sum, subtract);
    let zero = zero_flag(builder, &bits.sum);
    let (carry, half_carry) = carry_flags(builder, bits.carry, bits.half_carry, subtract);
    UnitFlags {
        carry,
        zero,
        negative: bits.sum[7],
        overflow,
        sign: builder.xor(bits.sum[7], overflow),
        half_carry,
    }
}

fn carry_flags(
    builder: &mut DigitalBuilder,
    carry: NetId,
    half_carry: NetId,
    subtract: NetId,
) -> (NetId, NetId) {
    (
        carry_flag(builder, carry, subtract),
        carry_flag(builder, half_carry, subtract),
    )
}
fn validate(first: &[NetId], second: &[NetId]) -> Result<(), SimError> {
    if first.len() != 8 || second.len() != 8 {
        return Err(SimError::InvalidNetlist(
            "AVR arithmetic expects 8-bit operands".to_string(),
        ));
    }
    Ok(())
}
fn add_bytes(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    carry_in: NetId,
) -> SumBits {
    let initial = adder_carry(builder, carry_in, subtract);
    let (sum, carry, half_carry) = ripple(builder, first, second, subtract, initial);
    SumBits {
        sum,
        carry,
        half_carry,
    }
}

fn ripple(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    initial: NetId,
) -> (Vec<NetId>, NetId, NetId) {
    let mut carry = initial;
    let mut half_carry = initial;
    let mut sum = Vec::with_capacity(8);
    for index in 0..8 {
        let (result, next) = ripple_step(builder, first[index], second[index], carry, subtract);
        if index == 3 {
            half_carry = next;
        }
        sum.push(result);
        carry = next;
    }
    (sum, carry, half_carry)
}

fn ripple_step(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry: NetId,
    subtract: NetId,
) -> (NetId, NetId) {
    let bit = effective_bit(builder, second, subtract);
    digital_full_adder(builder, first, bit, carry)
}

fn effective_bit(builder: &mut DigitalBuilder, bit: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(bit);
    builder.mux(bit, inverted, subtract)
}

fn adder_carry(builder: &mut DigitalBuilder, carry_in: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(carry_in);
    builder.mux(carry_in, inverted, subtract)
}

fn carry_flag(builder: &mut DigitalBuilder, value: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(value);
    builder.mux(value, inverted, subtract)
}

fn zero_flag(builder: &mut DigitalBuilder, result: &[NetId]) -> NetId {
    let any = or_all(builder, result);
    builder.not(any)
}

fn overflow_flag(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    result: &[NetId],
    subtract: NetId,
) -> NetId {
    let addition = addition_overflow(builder, first[7], second[7], result[7]);
    let subtraction = subtraction_overflow(builder, first[7], second[7], result[7]);
    builder.mux(addition, subtraction, subtract)
}

fn addition_overflow(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    result: NetId,
) -> NetId {
    let first_bar = builder.not(first);
    let second_bar = builder.not(second);
    let result_bar = builder.not(result);
    let same = and_all(builder, &[first, second, result_bar]);
    let low = and_all(builder, &[first_bar, second_bar, result]);
    or_all(builder, &[same, low])
}

fn subtraction_overflow(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    result: NetId,
) -> NetId {
    let first_bar = builder.not(first);
    let second_bar = builder.not(second);
    let result_bar = builder.not(result);
    let negative = and_all(builder, &[first, second_bar, result_bar]);
    let positive = and_all(builder, &[first_bar, second, result]);
    or_all(builder, &[negative, positive])
}
