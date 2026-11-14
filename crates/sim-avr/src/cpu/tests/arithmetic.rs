use super::*;

#[test]
fn runs_arithmetic_program() {
    let cpu = run(vec![ldi(16, 5), ldi(17, 3), reg_op(0x0C00, 16, 17), 0x9598]);
    assert_eq!(cpu.register(16), 8);
    assert!(cpu.is_halted());
}

#[test]
fn adiw_sets_positive_overflow() {
    let cpu = run(vec![ldi(24, 0xFF), ldi(25, 0x7F), 0x9601, 0x9598]);
    assert_eq!(cpu.register(24), 0x00);
    assert_eq!(cpu.register(25), 0x80);
    let flags = cpu.sreg();
    assert!(flags.negative);
    assert!(flags.overflow);
    assert!(!flags.carry);
    assert!(!flags.zero);
    assert!(!flags.sign);
}

#[test]
fn adiw_wraps_and_sets_carry() {
    let cpu = run(vec![ldi(24, 0xFF), ldi(25, 0xFF), 0x9601, 0x9598]);
    assert_eq!(cpu.register(24), 0x00);
    assert_eq!(cpu.register(25), 0x00);
    let flags = cpu.sreg();
    assert!(flags.carry);
    assert!(flags.zero);
    assert!(!flags.negative);
    assert!(!flags.overflow);
}

#[test]
fn sbiw_sets_borrow() {
    let cpu = run(vec![ldi(24, 0x00), ldi(25, 0x00), 0x9701, 0x9598]);
    assert_eq!(cpu.register(24), 0xFF);
    assert_eq!(cpu.register(25), 0xFF);
    let flags = cpu.sreg();
    assert!(flags.carry);
    assert!(flags.negative);
    assert!(!flags.zero);
    assert!(!flags.overflow);
    assert!(flags.sign);
}

#[test]
fn sbiw_sets_negative_overflow() {
    let cpu = run(vec![ldi(24, 0x00), ldi(25, 0x80), 0x9701, 0x9598]);
    assert_eq!(cpu.register(24), 0xFF);
    assert_eq!(cpu.register(25), 0x7F);
    let flags = cpu.sreg();
    assert!(!flags.carry);
    assert!(!flags.negative);
    assert!(flags.overflow);
    assert!(flags.sign);
}

#[test]
fn load_program_replaces_flash_and_resets() {
    let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598]).expect("cpu");
    cpu.run(10).expect("run");
    cpu.load_program(vec![ldi(16, 7), 0x9598]).expect("load");
    assert_eq!(cpu.program()[0], ldi(16, 7));
    cpu.run(10).expect("run");
    assert_eq!(cpu.register(16), 7);
    assert!(cpu.is_halted());
}

#[test]
fn load_program_rejects_empty() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    assert!(cpu.load_program(Vec::new()).is_err());
}

#[test]
fn load_program_runs_blink_pin() {
    let program = vec![
        0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
    ];
    let mut cpu = Cpu::new(vec![0x0000]).expect("cpu");
    cpu.load_program(program).expect("load");
    let config = PinElectricalConfig::default();
    let mut high = false;
    for _ in 0..2000 {
        cpu.step().expect("step");
        let state = cpu
            .pin_electrical(0, 5, ExternalPin::HighZ, &config)
            .expect("pin");
        high |= state.voltage > 2.5;
    }
    assert!(high);
}

#[test]
fn adc_uses_carry_flag() {
    let cpu = run(vec![
        ldi(16, 0xFF),
        ldi(17, 1),
        reg_op(0x0C00, 16, 17),
        ldi(18, 1),
        ldi(19, 0),
        reg_op(0x1C00, 18, 19),
        0x9598,
    ]);
    assert_eq!(cpu.register(18), 2);
}

#[test]
fn compare_and_branch_on_zero() {
    let cpu = run(vec![
        ldi(16, 0),
        reg_op(0x1800, 16, 16),
        branch(1, 1),
        ldi(17, 9),
        ldi(18, 7),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0);
    assert_eq!(cpu.register(18), 7);
}

#[test]
fn cpc_keeps_zero_clear_for_nonzero_low_byte() {
    let cpu = run(vec![
        ldi(24, 1),
        ldi(25, 0),
        ldi(22, 0),
        ldi(23, 0),
        reg_op(0x1400, 24, 22),
        reg_op(0x0400, 25, 23),
        0x9598,
    ]);
    assert!(!cpu.sreg().zero);
}

#[test]
fn sbc_keeps_zero_clear_for_nonzero_low_byte() {
    let cpu = run(vec![
        ldi(24, 1),
        ldi(25, 0),
        ldi(22, 0),
        ldi(23, 0),
        reg_op(0x0800, 24, 22),
        reg_op(0x0800, 25, 23),
        0x9598,
    ]);
    assert!(!cpu.sreg().zero);
}

#[test]
fn movw_shift_and_rotate() {
    let cpu = run(vec![
        ldi(26, 0x34),
        ldi(27, 0x12),
        0x01FD,
        one_reg(0x9406, 30),
        one_reg(0x9405, 31),
        one_reg(0x9407, 30),
        one_reg(0x9402, 30),
        0x9598,
    ]);
    assert_eq!(cpu.register(30), 0xD0);
    assert_eq!(cpu.register(31), 0x09);
}

#[test]
fn multiply_program() {
    let cpu = run(vec![ldi(16, 7), ldi(17, 6), reg_op(0x9C00, 16, 17), 0x9598]);
    assert_eq!(cpu.register(0), 42);
    assert_eq!(cpu.register(1), 0);
}

#[test]
fn jump_skips_instruction() {
    let cpu = run(vec![0x940C, 0x0003, ldi(16, 9), ldi(17, 4), 0x9598]);
    assert_eq!(cpu.register(16), 0);
    assert_eq!(cpu.register(17), 4);
}

#[test]
fn trace_records_pc_and_registers() {
    let mut cpu = Cpu::new(vec![ldi(16, 5), 0x9598]).expect("cpu");
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 1);
    assert_eq!(trace.registers[16], 5);
    assert!(!trace.halted);
    assert!(cpu.step().expect("step").expect("trace").halted);
}

#[test]
fn halted_core_returns_no_trace() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    assert!(cpu.step().expect("step").is_some());
    assert!(cpu.step().expect("step").is_none());
}

#[test]
fn logic_program() {
    let cpu = run(vec![
        ldi(16, 0xAA),
        ldi(17, 0xCC),
        reg_op(0x2000, 16, 17),
        immediate(0x6000, 16, 0x01),
        reg_op(0x2400, 16, 17),
        one_reg(0x9400, 16),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 0xBA);
}

#[test]
fn neg_inc_dec_program() {
    let cpu = run(vec![
        ldi(16, 5),
        one_reg(0x9401, 16),
        one_reg(0x9403, 16),
        one_reg(0x940A, 16),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 0xFB);
}

#[test]
fn subi_and_cpi_branch() {
    let cpu = run(vec![
        ldi(16, 10),
        immediate(0x5000, 16, 3),
        immediate(0x3000, 16, 7),
        branch(1, 1),
        ldi(17, 9),
        ldi(18, 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 7);
    assert_eq!(cpu.register(17), 0);
    assert_eq!(cpu.register(18), 4);
}
