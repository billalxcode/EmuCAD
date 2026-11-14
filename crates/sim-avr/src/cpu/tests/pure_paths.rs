use super::*;

#[test]
fn signed_multiply_uses_twos_complement() {
    let cpu = run(vec![ldi(16, 0xFF), ldi(17, 2), 0x0201, 0x9598]);
    assert_eq!(cpu.register(0), 0xFE);
    assert_eq!(cpu.register(1), 0xFF);
}

#[test]
fn mixed_multiply_uses_signed_first_operand() {
    let cpu = run(vec![ldi(16, 0xFF), ldi(17, 2), 0x0301, 0x9598]);
    assert_eq!(cpu.register(0), 0xFE);
    assert_eq!(cpu.register(1), 0xFF);
}

#[test]
fn fixed_point_multiply_shifts_left() {
    let cpu = run(vec![ldi(16, 0x80), ldi(17, 2), 0x0309, 0x9598]);
    assert_eq!(cpu.register(0), 0x00);
    assert_eq!(cpu.register(1), 0x02);
}

#[test]
fn shift_sets_carry_and_overflow() {
    let cpu = run(vec![ldi(16, 0x81), one_reg(0x9406, 16), 0x9598]);
    assert_eq!(cpu.register(16), 0x40);
    assert!(cpu.sreg().carry);
    assert!(cpu.sreg().overflow);
    assert!(cpu.sreg().sign);
}

#[test]
fn increment_sets_overflow_at_signed_boundary() {
    let cpu = run(vec![ldi(16, 0x7F), one_reg(0x9403, 16), 0x9598]);
    assert_eq!(cpu.register(16), 0x80);
    assert!(cpu.sreg().overflow);
    assert!(cpu.sreg().negative);
    assert!(!cpu.sreg().sign);
}

#[test]
fn decrement_sets_overflow_at_signed_boundary() {
    let cpu = run(vec![ldi(16, 0x80), one_reg(0x940A, 16), 0x9598]);
    assert_eq!(cpu.register(16), 0x7F);
    assert!(cpu.sreg().overflow);
    assert!(!cpu.sreg().negative);
    assert!(cpu.sreg().sign);
}
