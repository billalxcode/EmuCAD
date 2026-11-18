use super::*;

#[test]
fn rjmp_and_brbc() {
    let cpu = run(vec![
        ldi(16, 1),
        0xC001,
        ldi(17, 9),
        ldi(18, 4),
        branch_clear(1, 1),
        ldi(19, 9),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0);
    assert_eq!(cpu.register(18), 4);
    assert_eq!(cpu.register(19), 0);
}

#[test]
fn cpse_skips_equal_registers() {
    let cpu = run(vec![
        ldi(16, 4),
        ldi(17, 4),
        reg_op(0x1000, 16, 17),
        ldi(18, 9),
        ldi(19, 3),
        0x9598,
    ]);
    assert_eq!(cpu.register(18), 0);
    assert_eq!(cpu.register(19), 3);
}

#[test]
fn sbrc_and_sbrs_skip() {
    let cpu = run(vec![
        ldi(16, 4),
        bit_register(0xFE00, 16, 2),
        ldi(17, 9),
        bit_register(0xFC00, 16, 0),
        ldi(18, 9),
        ldi(19, 3),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0);
    assert_eq!(cpu.register(18), 0);
    assert_eq!(cpu.register(19), 3);
}

#[test]
fn io_in_out_and_bit_ops() {
    let cpu = run(vec![
        ldi(16, 0xAA),
        in_out(0xB800, 16, 0x10),
        in_out(0xB000, 17, 0x10),
        io_bit_op(0x9A00, 0x10, 0),
        io_bit_op(0x9800, 0x10, 1),
        in_out(0xB000, 18, 0x10),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0xAA);
    assert_eq!(cpu.register(18), 0xA9);
    assert_eq!(cpu.io(0x10), 0xA9);
}

#[test]
fn sbic_and_sbis_skip() {
    let cpu = run(vec![
        io_bit_op(0x9A00, 0x10, 2),
        io_bit_op(0x9B00, 0x10, 2),
        ldi(16, 9),
        io_bit_op(0x9800, 0x10, 2),
        io_bit_op(0x9900, 0x10, 2),
        ldi(17, 9),
        ldi(18, 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 0);
    assert_eq!(cpu.register(17), 0);
    assert_eq!(cpu.register(18), 4);
}

#[test]
fn sreg_flag_and_transfer_bits() {
    let cpu = run(vec![
        sreg_flag(0x9408, 0),
        sreg_flag(0x9488, 0),
        ldi(16, 1),
        bit_register(0xFA00, 16, 0),
        ldi(17, 0),
        bit_register(0xF800, 17, 3),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0x08);
    assert!(cpu.sreg().transfer);
    assert!(!cpu.sreg().carry);
}
