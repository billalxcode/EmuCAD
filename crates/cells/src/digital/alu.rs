//! Parameterized digital ALU.

use super::blocks::{decoder, full_adder, mux_tree, or_all};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Result and flags of the digital ALU.
pub struct DigitalAlu {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
    /// Result is zero.
    pub zero: NetId,
    /// Adder carry out.
    pub carry: NetId,
    /// Most significant result bit.
    pub negative: NetId,
    /// Adder carry out of bit 3 or the final bit for narrow widths.
    pub half_carry: NetId,
}

struct LogicBytes {
    and: Vec<NetId>,
    or: Vec<NetId>,
    xor: Vec<NetId>,
}

/// Builds an eight-bit ALU.
///
/// Operation select: 0 ADD, 1 SUB, 2 AND, 3 OR, 4 XOR, 5 pass B.
pub fn digital_alu(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
) -> Result<DigitalAlu, SimError> {
    digital_alu_width(builder, first, second, op, 8)
}

/// Builds an ALU with an explicit width from 1 to 64 bits.
///
/// Operation select: 0 ADD, 1 SUB, 2 AND, 3 OR, 4 XOR, 5 pass B.
pub fn digital_alu_width(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
    width: usize,
) -> Result<DigitalAlu, SimError> {
    validate_alu(first, second, op, width)?;
    let lines = decoder(builder, op)?;
    let subtract = lines[1];
    let (result, carry, half_carry) = alu_result(builder, first, second, op, subtract, width)?;
    Ok(finish_alu(builder, result, carry, half_carry))
}

fn alu_result(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
    subtract: NetId,
    width: usize,
) -> Result<(Vec<NetId>, NetId, NetId), SimError> {
    let (sum, raw_carry, raw_half) = add_bytes(builder, first, second, subtract, width);
    let logic = logic_bytes(builder, first, second, width);
    let result = select_result(builder, &sum, &logic, second, op, width)?;
    let carry = carry_flag(builder, raw_carry, subtract);
    let half_carry = carry_flag(builder, raw_half, subtract);
    Ok((result, carry, half_carry))
}

fn finish_alu(
    builder: &mut DigitalBuilder,
    result: Vec<NetId>,
    carry: NetId,
    half_carry: NetId,
) -> DigitalAlu {
    let any = or_all(builder, &result);
    let negative = result
        .last()
        .copied()
        .unwrap_or_else(|| builder.constant(false));
    DigitalAlu {
        zero: builder.not(any),
        negative,
        result,
        carry,
        half_carry,
    }
}

fn carry_flag(builder: &mut DigitalBuilder, value: NetId, subtract: NetId) -> NetId {
    let inverted = builder.not(value);
    builder.mux(value, inverted, subtract)
}

fn validate_alu(
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
    width: usize,
) -> Result<(), SimError> {
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "ALU width must be 1..=64".to_string(),
        ));
    }
    if first.len() != width || second.len() != width || op.len() != 3 {
        return Err(SimError::InvalidNetlist(
            "ALU operand width mismatch or opcode is not 3 bits".to_string(),
        ));
    }
    Ok(())
}

fn add_bytes(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    subtract: NetId,
    width: usize,
) -> (Vec<NetId>, NetId, NetId) {
    let (mut carry, mut half_carry) = (subtract, subtract);
    let half_index = if width > 3 { 3 } else { width - 1 };
    let mut sum = Vec::with_capacity(width);
    for index in 0..width {
        let (result, next) = add_bit(builder, first[index], second[index], carry, subtract);
        if index == half_index {
            half_carry = next;
        }
        sum.push(result);
        carry = next;
    }
    (sum, carry, half_carry)
}

fn add_bit(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry_in: NetId,
    subtract: NetId,
) -> (NetId, NetId) {
    let inverted = builder.not(second);
    let bit = builder.mux(second, inverted, subtract);
    full_adder(builder, first, bit, carry_in)
}

fn logic_bytes(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    width: usize,
) -> LogicBytes {
    let mut and = Vec::with_capacity(width);
    let mut or = Vec::with_capacity(width);
    let mut xor = Vec::with_capacity(width);
    for index in 0..width {
        and.push(builder.and(first[index], second[index]));
        or.push(builder.or(first[index], second[index]));
        xor.push(builder.xor(first[index], second[index]));
    }
    LogicBytes { and, or, xor }
}

fn select_result(
    builder: &mut DigitalBuilder,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    op: &[NetId],
    width: usize,
) -> Result<Vec<NetId>, SimError> {
    let zero = builder.constant(false);
    (0..width)
        .map(|bit| select_bit(builder, bit, sum, logic, pass, op, zero))
        .collect()
}

fn select_bit(
    builder: &mut DigitalBuilder,
    bit: usize,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    op: &[NetId],
    zero: NetId,
) -> Result<NetId, SimError> {
    let options = select_options(bit, sum, logic, pass, zero);
    mux_tree(builder, &options, op)
}

fn select_options(
    bit: usize,
    sum: &[NetId],
    logic: &LogicBytes,
    pass: &[NetId],
    zero: NetId,
) -> [NetId; 8] {
    [
        sum[bit],
        sum[bit],
        logic.and[bit],
        logic.or[bit],
        logic.xor[bit],
        pass[bit],
        zero,
        zero,
    ]
}

#[cfg(test)]
mod tests {
    use super::{DigitalAlu, digital_alu, digital_alu_width};
    use crate::digital::{DigitalBuilder, DigitalCircuit};
    use crate::test_support::{named_inputs, set_bits};

    fn fixture() -> (DigitalCircuit, DigitalAlu) {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", 8);
        let second = named_inputs(&mut builder, "b", 8);
        let op = named_inputs(&mut builder, "op", 3);
        let alu = digital_alu(&mut builder, &first, &second, &op).expect("alu");
        (builder.build(), alu)
    }

    fn evaluate(
        circuit: &mut DigitalCircuit,
        alu: &DigitalAlu,
        first: u64,
        second: u64,
        op: u64,
    ) -> (u64, bool, bool, bool, bool) {
        set_bits(circuit, "a", first, 8);
        set_bits(circuit, "b", second, 8);
        set_bits(circuit, "op", op, 3);
        circuit.settle().expect("settle");
        let result = crate::read_bits(circuit, &alu.result);
        (
            result,
            circuit.value(alu.zero),
            circuit.value(alu.carry),
            circuit.value(alu.negative),
            circuit.value(alu.half_carry),
        )
    }

    #[test]
    fn addition_and_subtraction() {
        let (mut circuit, alu) = fixture();
        assert_eq!(evaluate(&mut circuit, &alu, 5, 3, 0).0, 8);
        let (result, zero, carry, _, _) = evaluate(&mut circuit, &alu, 0xFF, 1, 0);
        assert_eq!((result, zero, carry), (0, true, true));
        assert_eq!(evaluate(&mut circuit, &alu, 9, 4, 1).0, 5);
        let (result, _, carry, negative, _) = evaluate(&mut circuit, &alu, 0, 1, 1);
        assert_eq!((result, carry, negative), (0xFF, true, true));
    }

    #[test]
    fn half_carry_tracks_bit_three() {
        let (mut circuit, alu) = fixture();
        let (_, _, _, _, half_carry) = evaluate(&mut circuit, &alu, 0x0F, 1, 0);
        assert!(half_carry);
        let (_, _, _, _, half_carry) = evaluate(&mut circuit, &alu, 0x0E, 1, 0);
        assert!(!half_carry);
    }

    #[test]
    fn logic_and_pass_operations() {
        let (mut circuit, alu) = fixture();
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 2).0, 0x88);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 3).0, 0xEE);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 4).0, 0x66);
        assert_eq!(evaluate(&mut circuit, &alu, 0xAA, 0xCC, 5).0, 0xCC);
    }

    #[test]
    fn zero_and_negative_flags() {
        let (mut circuit, alu) = fixture();
        let (_, zero, _, negative, _) = evaluate(&mut circuit, &alu, 0xFF, 0xFF, 4);
        assert!(zero && !negative);
        let (_, zero, _, negative, _) = evaluate(&mut circuit, &alu, 0, 0x80, 5);
        assert!(!zero && negative);
    }

    fn wide_fixture(width: usize) -> (DigitalCircuit, DigitalAlu) {
        let mut builder = DigitalBuilder::new();
        let first = named_inputs(&mut builder, "a", width);
        let second = named_inputs(&mut builder, "b", width);
        let op = named_inputs(&mut builder, "op", 3);
        let alu = digital_alu_width(&mut builder, &first, &second, &op, width).expect("alu");
        (builder.build(), alu)
    }

    fn evaluate_wide(
        circuit: &mut DigitalCircuit,
        alu: &DigitalAlu,
        first: u64,
        second: u64,
        op: u64,
        width: usize,
    ) -> (u64, bool, bool, bool) {
        set_bits(circuit, "a", first, width);
        set_bits(circuit, "b", second, width);
        set_bits(circuit, "op", op, 3);
        circuit.settle().expect("settle");
        (
            crate::read_bits(circuit, &alu.result),
            circuit.value(alu.zero),
            circuit.value(alu.carry),
            circuit.value(alu.negative),
        )
    }

    #[test]
    fn wide_arithmetic_uses_bit_31() {
        let (mut circuit, alu) = wide_fixture(32);
        let (result, zero, carry, _) = evaluate_wide(&mut circuit, &alu, 0xFFFF_FFFF, 1, 0, 32);
        assert_eq!((result, zero, carry), (0, true, true));
        let (result, zero, carry, negative) = evaluate_wide(&mut circuit, &alu, 0, 1, 1, 32);
        assert_eq!(
            (result, zero, carry, negative),
            (0xFFFF_FFFF, false, true, true)
        );
    }

    #[test]
    fn wide_logic_uses_all_bits() {
        let (mut circuit, alu) = wide_fixture(32);
        let (result, _, _, _) = evaluate_wide(&mut circuit, &alu, 0xF0F0_F0F0, 0x0FF0_0FF0, 2, 32);
        assert_eq!(result, 0x00F0_00F0);
        let (result, _, _, _) = evaluate_wide(&mut circuit, &alu, 0xF0F0_F0F0, 0x0FF0_0FF0, 3, 32);
        assert_eq!(result, 0xFFF0_FFF0);
    }
}
