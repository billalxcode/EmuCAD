use super::{
    ArithmeticUnit, CompareUnit, LogicUnit, arithmetic_unit, compare_unit, decrement_unit,
    increment_unit, logic_unit, negate_unit, select_operand,
};
use crate::test_support::{named_inputs, set_bits};
use sim_cells::{
    DigitalBuilder, DigitalCircuit, NetId, add_with_carry, read_bits, sub_with_borrow,
};
use sim_core::SimError;

#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    value: u8,
    carry: bool,
    zero: bool,
    negative: bool,
    overflow: bool,
    sign: bool,
    half_carry: bool,
}

fn fixture() -> (DigitalCircuit, ArithmeticUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let second = named_inputs(&mut builder, "b", 8);
    let subtract = builder.input("sub");
    let carry = builder.input("cin");
    let unit = arithmetic_unit(&mut builder, &first, &second, subtract, carry).expect("unit");
    (builder.build(), unit)
}

fn evaluate(
    circuit: &mut DigitalCircuit,
    unit: &ArithmeticUnit,
    first: u8,
    second: u8,
    subtract: bool,
    carry_in: bool,
) -> Outcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", u64::from(second), 8);
    circuit.set_input("sub", subtract).expect("sub");
    circuit.set_input("cin", carry_in).expect("cin");
    circuit.settle().expect("settle");
    read_outcome(circuit, unit)
}

fn read_outcome(circuit: &DigitalCircuit, unit: &ArithmeticUnit) -> Outcome {
    Outcome {
        value: read_bits(circuit, &unit.result) as u8,
        carry: circuit.value(unit.carry),
        zero: circuit.value(unit.zero),
        negative: circuit.value(unit.negative),
        overflow: circuit.value(unit.overflow),
        sign: circuit.value(unit.sign),
        half_carry: circuit.value(unit.half_carry),
    }
}

fn addition(first: u8, second: u8, carry_in: bool) -> Outcome {
    let result = add_with_carry(first, second, carry_in);
    Outcome {
        value: result.value,
        carry: result.flags.carry,
        zero: result.flags.zero,
        negative: result.flags.negative,
        overflow: result.flags.overflow,
        sign: result.flags.sign,
        half_carry: result.flags.half_carry,
    }
}

fn subtraction(first: u8, second: u8, borrow: bool) -> Outcome {
    let result = sub_with_borrow(first, second, borrow);
    Outcome {
        value: result.value,
        carry: result.flags.carry,
        zero: result.flags.zero,
        negative: result.flags.negative,
        overflow: result.flags.overflow,
        sign: result.flags.sign,
        half_carry: result.flags.half_carry,
    }
}

#[test]
fn addition_matches_reference() {
    let (mut circuit, unit) = fixture();
    let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
    for first in 0..=u8::MAX {
        for second in values {
            for carry in [false, true] {
                let actual = evaluate(&mut circuit, &unit, first, second, false, carry);
                assert_eq!(actual, addition(first, second, carry));
            }
        }
    }
}

#[test]
fn subtraction_matches_reference() {
    let (mut circuit, unit) = fixture();
    let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
    for first in 0..=u8::MAX {
        for second in values {
            for borrow in [false, true] {
                let actual = evaluate(&mut circuit, &unit, first, second, true, borrow);
                assert_eq!(actual, subtraction(first, second, borrow));
            }
        }
    }
}

#[test]
fn rejects_wrong_operand_width() {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let second = named_inputs(&mut builder, "b", 7);
    let subtract = builder.input("sub");
    let carry = builder.input("cin");
    assert!(arithmetic_unit(&mut builder, &first, &second, subtract, carry).is_err());
}

#[derive(Debug, PartialEq, Eq)]
struct LogicOutcome {
    value: u8,
    zero: bool,
    negative: bool,
    sign: bool,
}

fn logic_fixture() -> (DigitalCircuit, LogicUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let second = named_inputs(&mut builder, "b", 8);
    let op = named_inputs(&mut builder, "op", 2);
    let unit = logic_unit(&mut builder, &first, &second, &op).expect("logic unit");
    (builder.build(), unit)
}

fn evaluate_logic(
    circuit: &mut DigitalCircuit,
    unit: &LogicUnit,
    first: u8,
    second: u8,
    op: u8,
) -> LogicOutcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", u64::from(second), 8);
    set_bits(circuit, "op", u64::from(op), 2);
    circuit.settle().expect("settle");
    LogicOutcome {
        value: read_bits(circuit, &unit.result) as u8,
        zero: circuit.value(unit.zero),
        negative: circuit.value(unit.negative),
        sign: circuit.value(unit.sign),
    }
}

fn logic_reference(op: u8, first: u8, second: u8) -> LogicOutcome {
    let value = match op {
        0 => first & second,
        1 => first | second,
        2 => first ^ second,
        _ => !first,
    };
    LogicOutcome {
        value,
        zero: value == 0,
        negative: value & 0x80 != 0,
        sign: value & 0x80 != 0,
    }
}

#[test]
fn logic_operations_match_reference() {
    let (mut circuit, unit) = logic_fixture();
    let values = [0, 1, 0x0F, 0x55, 0x80, 0xAA, 0xFF];
    for first in 0..=u8::MAX {
        for second in values {
            for op in 0..4 {
                let actual = evaluate_logic(&mut circuit, &unit, first, second, op);
                assert_eq!(actual, logic_reference(op, first, second));
            }
        }
    }
}

#[test]
fn logic_rejects_wrong_width() {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let second = named_inputs(&mut builder, "b", 8);
    let op = named_inputs(&mut builder, "op", 3);
    assert!(logic_unit(&mut builder, &first, &second, &op).is_err());
}

fn immediate_arithmetic_fixture() -> (DigitalCircuit, ArithmeticUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let register = named_inputs(&mut builder, "b", 8);
    let immediate = named_inputs(&mut builder, "k", 8);
    let use_immediate = builder.input("imm");
    let selected =
        select_operand(&mut builder, &register, &immediate, use_immediate).expect("select");
    let subtract = builder.input("sub");
    let carry = builder.input("cin");
    let unit = arithmetic_unit(&mut builder, &first, &selected, subtract, carry).expect("unit");
    (builder.build(), unit)
}

fn evaluate_immediate(
    circuit: &mut DigitalCircuit,
    unit: &ArithmeticUnit,
    first: u8,
    immediate: u8,
    carry_in: bool,
) -> Outcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", 0, 8);
    set_bits(circuit, "k", u64::from(immediate), 8);
    circuit.set_input("imm", true).expect("imm");
    circuit.set_input("sub", true).expect("sub");
    circuit.set_input("cin", carry_in).expect("cin");
    circuit.settle().expect("settle");
    read_outcome(circuit, unit)
}

#[test]
fn subi_and_sbci_match_reference() {
    let (mut circuit, unit) = immediate_arithmetic_fixture();
    let values = [0, 1, 0x0F, 0x80, 0xFF];
    for first in 0..=u8::MAX {
        for immediate in values {
            for carry in [false, true] {
                let actual = evaluate_immediate(&mut circuit, &unit, first, immediate, carry);
                assert_eq!(actual, subtraction(first, immediate, carry));
            }
        }
    }
}

fn immediate_logic_fixture() -> (DigitalCircuit, LogicUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let register = named_inputs(&mut builder, "b", 8);
    let immediate = named_inputs(&mut builder, "k", 8);
    let use_immediate = builder.input("imm");
    let selected =
        select_operand(&mut builder, &register, &immediate, use_immediate).expect("select");
    let op = named_inputs(&mut builder, "op", 2);
    let unit = logic_unit(&mut builder, &first, &selected, &op).expect("unit");
    (builder.build(), unit)
}

fn evaluate_immediate_logic(
    circuit: &mut DigitalCircuit,
    unit: &LogicUnit,
    first: u8,
    immediate: u8,
    op: u8,
) -> LogicOutcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", 0, 8);
    set_bits(circuit, "k", u64::from(immediate), 8);
    set_bits(circuit, "op", u64::from(op), 2);
    circuit.set_input("imm", true).expect("imm");
    circuit.settle().expect("settle");
    LogicOutcome {
        value: read_bits(circuit, &unit.result) as u8,
        zero: circuit.value(unit.zero),
        negative: circuit.value(unit.negative),
        sign: circuit.value(unit.sign),
    }
}

#[test]
fn andi_and_ori_match_reference() {
    let (mut circuit, unit) = immediate_logic_fixture();
    let values = [0, 1, 0x0F, 0x55, 0x80, 0xAA, 0xFF];
    for first in 0..=u8::MAX {
        for immediate in values {
            for op in [0, 1] {
                let actual = evaluate_immediate_logic(&mut circuit, &unit, first, immediate, op);
                assert_eq!(actual, logic_reference(op, first, immediate));
            }
        }
    }
}

type UnitBuilder = fn(&mut DigitalBuilder, &[NetId]) -> Result<ArithmeticUnit, SimError>;

fn update_fixture(builder: UnitBuilder) -> (DigitalCircuit, ArithmeticUnit) {
    let mut digital = DigitalBuilder::new();
    let input = named_inputs(&mut digital, "a", 8);
    let unit = builder(&mut digital, &input).expect("unit");
    (digital.build(), unit)
}

fn evaluate_update(circuit: &mut DigitalCircuit, unit: &ArithmeticUnit, value: u8) -> Outcome {
    set_bits(circuit, "a", u64::from(value), 8);
    circuit.settle().expect("settle");
    read_outcome(circuit, unit)
}

#[test]
fn neg_matches_reference() {
    let (mut circuit, unit) = update_fixture(negate_unit);
    for value in 0..=u8::MAX {
        let actual = evaluate_update(&mut circuit, &unit, value);
        assert_eq!(actual, subtraction(0, value, false));
    }
}

#[test]
fn inc_matches_reference() {
    let (mut circuit, unit) = update_fixture(increment_unit);
    for value in 0..=u8::MAX {
        let actual = evaluate_update(&mut circuit, &unit, value);
        let expected = value.wrapping_add(1);
        assert_eq!(actual.value, expected);
        assert_eq!(actual.zero, expected == 0);
        assert_eq!(actual.negative, expected & 0x80 != 0);
        assert_eq!(actual.overflow, expected == 0x80);
        assert_eq!(actual.sign, actual.negative ^ actual.overflow);
    }
}

#[test]
fn dec_matches_reference() {
    let (mut circuit, unit) = update_fixture(decrement_unit);
    for value in 0..=u8::MAX {
        let actual = evaluate_update(&mut circuit, &unit, value);
        let expected = value.wrapping_sub(1);
        assert_eq!(actual.value, expected);
        assert_eq!(actual.zero, expected == 0);
        assert_eq!(actual.negative, expected & 0x80 != 0);
        assert_eq!(actual.overflow, expected == 0x7F);
        assert_eq!(actual.sign, actual.negative ^ actual.overflow);
    }
}

fn compare_fixture() -> (DigitalCircuit, CompareUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let second = named_inputs(&mut builder, "b", 8);
    let carry = builder.input("cin");
    let unit = compare_unit(&mut builder, &first, &second, carry).expect("compare");
    (builder.build(), unit)
}

fn compare_outcome(circuit: &DigitalCircuit, unit: &CompareUnit) -> Outcome {
    Outcome {
        value: 0,
        carry: circuit.value(unit.carry),
        zero: circuit.value(unit.zero),
        negative: circuit.value(unit.negative),
        overflow: circuit.value(unit.overflow),
        sign: circuit.value(unit.sign),
        half_carry: circuit.value(unit.half_carry),
    }
}

fn evaluate_compare(
    circuit: &mut DigitalCircuit,
    unit: &CompareUnit,
    first: u8,
    second: u8,
    carry_in: bool,
) -> Outcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", u64::from(second), 8);
    circuit.set_input("cin", carry_in).expect("cin");
    circuit.settle().expect("settle");
    compare_outcome(circuit, unit)
}

fn assert_same_flags(actual: &Outcome, expected: &Outcome) {
    assert_eq!(actual.carry, expected.carry);
    assert_eq!(actual.zero, expected.zero);
    assert_eq!(actual.negative, expected.negative);
    assert_eq!(actual.overflow, expected.overflow);
    assert_eq!(actual.sign, expected.sign);
    assert_eq!(actual.half_carry, expected.half_carry);
}

#[test]
fn cp_and_cpc_match_reference() {
    let (mut circuit, unit) = compare_fixture();
    let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
    for first in 0..=u8::MAX {
        for second in values {
            for carry in [false, true] {
                let actual = evaluate_compare(&mut circuit, &unit, first, second, carry);
                let expected = subtraction(first, second, carry);
                assert_same_flags(&actual, &expected);
            }
        }
    }
}

fn immediate_compare_fixture() -> (DigitalCircuit, CompareUnit) {
    let mut builder = DigitalBuilder::new();
    let first = named_inputs(&mut builder, "a", 8);
    let register = named_inputs(&mut builder, "b", 8);
    let immediate = named_inputs(&mut builder, "k", 8);
    let use_immediate = builder.input("imm");
    let selected =
        select_operand(&mut builder, &register, &immediate, use_immediate).expect("select");
    let carry = builder.constant(false);
    let unit = compare_unit(&mut builder, &first, &selected, carry).expect("compare");
    (builder.build(), unit)
}

fn evaluate_immediate_compare(
    circuit: &mut DigitalCircuit,
    unit: &CompareUnit,
    first: u8,
    immediate: u8,
) -> Outcome {
    set_bits(circuit, "a", u64::from(first), 8);
    set_bits(circuit, "b", 0, 8);
    set_bits(circuit, "k", u64::from(immediate), 8);
    circuit.set_input("imm", true).expect("imm");
    circuit.settle().expect("settle");
    compare_outcome(circuit, unit)
}

#[test]
fn cpi_matches_reference() {
    let (mut circuit, unit) = immediate_compare_fixture();
    let values = [0, 1, 0x0F, 0x10, 0x7F, 0x80, 0xFF];
    for first in 0..=u8::MAX {
        for immediate in values {
            let actual = evaluate_immediate_compare(&mut circuit, &unit, first, immediate);
            let expected = subtraction(first, immediate, false);
            assert_same_flags(&actual, &expected);
        }
    }
}
