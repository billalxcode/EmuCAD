use super::Cpu;
use crate::pin::{ExternalPin, PinElectricalConfig};

fn ldi(register: u16, value: u16) -> u16 {
    0xE000 | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
}

fn reg_op(base: u16, destination: u16, source: u16) -> u16 {
    base | ((source & 0x10) << 5) | ((destination & 0x1F) << 4) | (source & 0x0F)
}

fn one_reg(base: u16, register: u16) -> u16 {
    base | ((register & 0x1F) << 4)
}

fn branch(flag: u16, offset: i8) -> u16 {
    0xF000 | (((offset as u16) & 0x7F) << 3) | flag
}

fn branch_clear(flag: u16, offset: i8) -> u16 {
    0xF400 | (((offset as u16) & 0x7F) << 3) | flag
}

fn immediate(base: u16, register: u16, value: u16) -> u16 {
    base | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
}

fn in_out(base: u16, register: u16, address: u16) -> u16 {
    base | (((address >> 4) & 3) << 9) | ((register & 0x1F) << 4) | (address & 0x0F)
}

fn io_bit_op(base: u16, address: u16, bit: u16) -> u16 {
    base | ((address & 0x1F) << 3) | (bit & 0x07)
}

fn bit_register(base: u16, register: u16, bit: u16) -> u16 {
    base | ((register & 0x1F) << 4) | (bit & 0x07)
}

fn sreg_flag(base: u16, flag: u16) -> u16 {
    base | ((flag & 0x07) << 4)
}

fn run(program: Vec<u16>) -> Cpu {
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.run(100).expect("run");
    cpu
}

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

#[test]
fn indirect_load_store_with_post_and_pre() {
    let cpu = run(vec![
        ldi(26, 0x00),
        ldi(27, 0x01),
        ldi(16, 0x55),
        0x920D | (16 << 4),
        0x900E | (17 << 4),
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0100), 0x55);
    assert_eq!(cpu.register(17), 0x55);
    assert_eq!(cpu.register(26), 0x00);
    assert_eq!(cpu.register(27), 0x01);
}

#[test]
fn ldd_and_std_displacement() {
    let cpu = run(vec![
        ldi(28, 0x00),
        ldi(29, 0x02),
        ldi(16, 0x66),
        0x8208 | (16 << 4) | 5,
        0x8008 | (17 << 4) | 5,
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0205), 0x66);
    assert_eq!(cpu.register(17), 0x66);
}

#[test]
fn lds_and_sts_direct() {
    let cpu = run(vec![
        ldi(16, 0x77),
        0x9200 | (16 << 4),
        0x0300,
        0x9000 | (17 << 4),
        0x0300,
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0300), 0x77);
    assert_eq!(cpu.register(17), 0x77);
}

#[test]
fn lpm_reads_program_byte() {
    let first = ldi(30, 0);
    let cpu = run(vec![first, ldi(16, 0), 0x9104, 0x9598]);
    assert_eq!(cpu.register(16), first as u8);
}

#[test]
fn lpm_post_increment_advances_z() {
    let first = ldi(30, 0);
    let cpu = run(vec![first, 0x9105, 0x9005 | (17 << 4), 0x9598]);
    assert_eq!(cpu.register(16), first as u8);
    assert_eq!(cpu.register(17), (first >> 8) as u8);
    assert_eq!(cpu.register(30), 2);
}

#[test]
fn push_and_pop_use_stack() {
    let cpu = run(vec![
        ldi(16, 0x5A),
        0x920F | (16 << 4),
        ldi(16, 0),
        0x900F | (17 << 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0x5A);
    assert_eq!(cpu.sp(), 0x08FF);
}

#[test]
fn rcall_and_ret_return_to_caller() {
    let cpu = run(vec![
        0xD003,
        ldi(16, 1),
        0x9598,
        ldi(17, 9),
        ldi(17, 7),
        0x9508,
    ]);
    assert_eq!(cpu.register(16), 1);
    assert_eq!(cpu.register(17), 7);
    assert_eq!(cpu.sp(), 0x08FF);
}

#[test]
fn call_and_ret_use_two_word_target() {
    let cpu = run(vec![0x940E, 0x0004, ldi(16, 1), 0x9598, ldi(17, 7), 0x9508]);
    assert_eq!(cpu.register(16), 1);
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn reti_sets_interrupt_flag() {
    let cpu = run(vec![0xD002, 0x9598, 0x0000, 0x9518]);
    assert!(cpu.sreg().interrupt);
}

#[test]
fn ijmp_uses_z_pointer() {
    let cpu = run(vec![
        ldi(30, 4),
        ldi(31, 0),
        0x9409,
        ldi(16, 9),
        ldi(17, 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 0);
    assert_eq!(cpu.register(17), 4);
}

#[test]
fn sreg_is_visible_through_io() {
    let cpu = run(vec![
        ldi(16, 0xFF),
        ldi(17, 1),
        reg_op(0x0C00, 16, 17),
        in_out(0xB000, 18, 0x3F),
        ldi(19, 0x80),
        in_out(0xB800, 19, 0x3F),
        0x9598,
    ]);
    assert_eq!(cpu.register(18) & 0x03, 0x03);
    assert!(cpu.sreg().interrupt);
}

#[test]
fn sleep_sets_sleeping_state() {
    let mut cpu = Cpu::new(vec![0x9588, ldi(16, 5), 0x9598]).expect("cpu");
    let trace = cpu.step().expect("step").expect("trace");
    assert!(!trace.halted);
    assert!(cpu.is_sleeping());
    assert!(cpu.step().expect("step").is_none());
}

#[test]
fn sleep_wakes_on_interrupt() {
    let mut cpu = Cpu::new(vec![0x9588, 0xCFFF, 0x0000, 0x0000, ldi(17, 7), 0x9518]).expect("cpu");
    cpu.request_interrupt(2).expect("interrupt");
    cpu.step().expect("step");
    assert!(cpu.is_sleeping());
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 5);
    assert_eq!(cpu.register(17), 7);
    assert!(!cpu.is_sleeping());
}

#[test]
fn wdr_is_noop() {
    let cpu = run(vec![0x95A8, ldi(16, 5), 0x9598]);
    assert_eq!(cpu.register(16), 5);
}

#[test]
fn reset_clears_core_state() {
    let mut cpu = Cpu::new(vec![ldi(16, 9), 0x9598]).expect("cpu");
    cpu.step().expect("step");
    cpu.reset().expect("reset");
    assert_eq!(cpu.pc(), 0);
    assert_eq!(cpu.register(16), 0);
    assert_eq!(cpu.sreg(), super::Sreg::default());
    assert_eq!(cpu.sp(), 0x08FF);
    assert!(!cpu.is_halted());
}

#[test]
fn sreg_byte_round_trip() {
    let sreg = super::Sreg {
        carry: true,
        zero: true,
        negative: true,
        overflow: true,
        sign: true,
        half_carry: true,
        transfer: true,
        interrupt: true,
    };
    assert_eq!(super::Sreg::from_byte(sreg.to_byte()), sreg);
}

#[test]
fn vector_table_matches_datasheet() {
    assert_eq!(super::VECTORS.len(), 26);
    assert_eq!(super::VECTORS[0], 0x0000);
    assert_eq!(super::VECTORS[1], 0x0002);
    assert_eq!(super::VECTORS[25], 0x0032);
}

#[test]
fn interrupt_entry_and_reti() {
    let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598, ldi(17, 7), 0x9518]).expect("cpu");
    cpu.step().expect("step");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    cpu.request_interrupt(1).expect("interrupt");
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 3);
    assert_eq!(cpu.register(17), 7);
    assert!(!cpu.sreg().interrupt);
    assert_eq!(cpu.sp(), 0x08FD);
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 1);
    assert!(cpu.sreg().interrupt);
}

#[test]
fn sei_delays_interrupt_until_after_next_instruction() {
    let mut cpu = Cpu::new(vec![
        sreg_flag(0x9408, 7),
        ldi(16, 1),
        0xCFFF,
        0x0000,
        ldi(17, 7),
        0x9518,
    ])
    .expect("cpu");
    cpu.request_interrupt(2).expect("interrupt");
    cpu.step().expect("step");
    assert!(!cpu.sreg().interrupt);
    cpu.step().expect("step");
    assert!(cpu.sreg().interrupt);
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 5);
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn interrupt_rejects_bad_vector() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    assert!(cpu.request_interrupt(26).is_err());
}

#[test]
fn instruction_and_total_cycles() {
    let mut cpu = Cpu::new(vec![
        ldi(16, 1),
        ldi(17, 2),
        reg_op(0x0C00, 16, 17),
        0xC001,
        ldi(18, 9),
        0x9598,
    ])
    .expect("cpu");
    let traces = cpu.run(10).expect("run");
    assert_eq!(traces.len(), 5);
    assert_eq!(cpu.cycles(), 6);
    assert_eq!(traces[4].total_cycles, 6);
}

#[test]
fn taken_branch_costs_two_cycles() {
    let mut cpu = Cpu::new(vec![
        ldi(16, 0),
        reg_op(0x1800, 16, 16),
        branch(1, 1),
        ldi(17, 9),
        ldi(18, 7),
        0x9598,
    ])
    .expect("cpu");
    let traces = cpu.run(10).expect("run");
    assert_eq!(traces[2].cycles, 2);
}

#[test]
fn skip_over_two_word_instruction_costs_three() {
    let mut cpu = Cpu::new(vec![
        ldi(16, 1),
        ldi(17, 1),
        reg_op(0x1000, 16, 17),
        0x9000 | (18 << 4),
        0x0100,
        0x9598,
    ])
    .expect("cpu");
    let traces = cpu.run(10).expect("run");
    assert_eq!(traces[2].cycles, 3);
}

#[test]
fn interrupt_entry_adds_four_cycles() {
    let mut cpu = Cpu::new(vec![ldi(16, 1), 0x9598, ldi(17, 7), 0x9518]).expect("cpu");
    cpu.step().expect("step");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    cpu.request_interrupt(1).expect("interrupt");
    let before = cpu.cycles();
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.cycles, 1);
    assert_eq!(cpu.cycles() - before, 5);
}

#[test]
fn gpio_output_and_pin_read() {
    let cpu = run(vec![
        ldi(16, 0x20),
        in_out(0xB800, 16, 0x04),
        in_out(0xB800, 16, 0x05),
        in_out(0xB000, 17, 0x03),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0x20);
    assert_eq!(cpu.io(0x04), 0x20);
    assert_eq!(cpu.io(0x05), 0x20);
}

#[test]
fn gpio_input_reads_external_pins() {
    let mut cpu = Cpu::new(vec![in_out(0xB000, 17, 0x03), 0x9598]).expect("cpu");
    cpu.set_pin_input(0, 0xAA).expect("pin");
    cpu.run(10).expect("run");
    assert_eq!(cpu.register(17), 0xAA);
}

#[test]
fn gpio_pull_up_tracks_port_register() {
    let cpu = run(vec![ldi(16, 0xFF), in_out(0xB800, 16, 0x05), 0x9598]);
    assert_eq!(cpu.pull_up(0), 0xFF);
}

fn pin_change_cpu() -> Cpu {
    Cpu::new(vec![
        ldi(16, 1),
        0xCFFF,
        0x0000,
        0x0000,
        0x0000,
        0x0000,
        ldi(17, 7),
        0x9518,
    ])
    .expect("cpu")
}

#[test]
fn pin_change_requests_interrupt() {
    let mut cpu = pin_change_cpu();
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    cpu.set_pin_change_mask(0, 0x01).expect("mask");
    cpu.enable_pin_change(0, true).expect("enable");
    cpu.set_pin_input(0, 0x01).expect("pin");
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 7);
    assert_eq!(cpu.register(17), 7);
    assert_eq!(cpu.pin_change_flags(), 0);
}

#[test]
fn pin_change_ignores_masked_and_disabled_bits() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    cpu.set_pin_change_mask(0, 0x02).expect("mask");
    cpu.enable_pin_change(0, true).expect("enable");
    cpu.set_pin_input(0, 0x01).expect("pin");
    assert_eq!(cpu.pin_change_flags(), 0);
    cpu.set_pin_change_mask(0, 0x01).expect("mask");
    cpu.enable_pin_change(0, false).expect("disable");
    cpu.set_pin_input(0, 0x00).expect("pin");
    assert_eq!(cpu.pin_change_flags(), 0);
}

#[test]
fn pin_change_flag_can_be_cleared() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    cpu.set_pin_change_mask(0, 0x01).expect("mask");
    cpu.enable_pin_change(0, true).expect("enable");
    cpu.set_pin_input(0, 0x01).expect("pin");
    assert_eq!(cpu.pin_change_flags(), 1);
    cpu.clear_pin_change_flag(0);
    assert_eq!(cpu.pin_change_flags(), 0);
}

#[test]
fn timer0_counts_cpu_cycles() {
    let cpu = run(vec![
        ldi(16, 1),
        in_out(0xB800, 16, 0x25),
        0x0000,
        0x0000,
        0x0000,
        0x0000,
        0x0000,
        in_out(0xB000, 17, 0x26),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 6);
}

#[test]
fn timer0_sets_overflow_flag() {
    let mut cpu = Cpu::new(vec![
        ldi(16, 1),
        in_out(0xB800, 16, 0x25),
        ldi(18, 255),
        one_reg(0x940A, 18),
        0xF7F1,
        in_out(0xB000, 17, 0x35),
        0x9598,
    ])
    .expect("cpu");
    cpu.run(2000).expect("run");
    assert_eq!(cpu.register(17) & 0x01, 0x01);
}

#[test]
fn timer0_sets_compare_flag_in_ctc() {
    let cpu = run(vec![
        ldi(16, 3),
        in_out(0xB800, 16, 0x27),
        ldi(16, 2),
        in_out(0xB800, 16, 0x24),
        ldi(16, 1),
        in_out(0xB800, 16, 0x25),
        0x0000,
        0x0000,
        0x0000,
        in_out(0xB000, 17, 0x35),
        0x9598,
    ]);
    assert_eq!(cpu.register(17) & 0x02, 0x02);
}

fn timer0_overflow_program() -> Vec<u16> {
    let mut program = vec![0u16; 40];
    program[0] = ldi(16, 1);
    program[1] = in_out(0xB800, 16, 0x25);
    program[2] = ldi(16, 1);
    program[3] = in_out(0xB800, 16, 0x2E);
    program[4] = ldi(18, 255);
    program[5] = one_reg(0x940A, 18);
    program[6] = 0xF7F1;
    program[7] = 0xCFFF;
    program[32] = ldi(17, 7);
    program[33] = 0x9518;
    program
}

#[test]
fn timer0_overflow_interrupt_enters_vector() {
    let mut cpu = Cpu::new(timer0_overflow_program()).expect("cpu");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    for _ in 0..2000 {
        cpu.step().expect("step");
        if cpu.register(17) == 7 {
            break;
        }
    }
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn timer0_overflow_vector_clears_flag() {
    let mut cpu = Cpu::new(timer0_overflow_program()).expect("cpu");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    for _ in 0..2000 {
        cpu.step().expect("step");
        if cpu.register(17) == 7 {
            break;
        }
    }
    assert_eq!(cpu.register(17), 7);
    assert_eq!(cpu.io(0x35) & 0x01, 0);
}
#[test]
fn timer0_pwm_output_from_cpu() {
    let mut cpu = Cpu::new(vec![
        ldi(16, 2),
        in_out(0xB800, 16, 0x27),
        ldi(16, 0x83),
        in_out(0xB800, 16, 0x24),
        ldi(16, 1),
        in_out(0xB800, 16, 0x25),
        0x0000,
        0x9598,
    ])
    .expect("cpu");
    for _ in 0..6 {
        cpu.step().expect("step");
    }
    assert!(cpu.pwm_a());
    cpu.step().expect("nop");
    assert!(!cpu.pwm_a());
}

#[test]
fn uart_registers_drive_transmitter() {
    let mut cpu = run(uart_control_program());
    assert!(!cpu.uart().tx_ready());
    cpu.uart().tick(200);
    assert!(cpu.uart().tx_complete());
    assert_eq!(cpu.uart().tx_byte(), 0x41);
    assert_eq!(cpu.io(0xC0) & 0x60, 0x60);
}

fn uart_control_program() -> Vec<u16> {
    vec![
        ldi(16, 0x08),
        0x9300,
        0x00C1,
        ldi(16, 0x00),
        0x9300,
        0x00C4,
        0x9300,
        0x00C5,
        ldi(16, 0x41),
        0x9300,
        0x00C6,
        0x9598,
    ]
}

#[test]
fn uart_data_read_clears_rxc() {
    let mut cpu = Cpu::new(vec![0x9100, 0x00C6, 0x9598]).expect("cpu");
    cpu.uart().receive(0x5A);
    assert_eq!(cpu.io(0xC0) & 0x80, 0x80);
    assert_eq!(cpu.io(0xC6), 0x5A);
    cpu.run(10).expect("run");
    assert_eq!(cpu.register(16), 0x5A);
    assert_eq!(cpu.io(0xC0) & 0x80, 0);
}

#[test]
fn uart_status_write_clears_txc() {
    let mut cpu = Cpu::new(vec![ldi(16, 0x40), 0x9300, 0x00C0, 0x9598]).expect("cpu");
    cpu.uart().write_control(1, 0x08);
    cpu.uart().write_tx(0x41);
    cpu.uart().tick(200);
    assert!(cpu.uart().tx_complete());
    cpu.run(10).expect("run");
    assert!(!cpu.uart().tx_complete());
}

#[test]
fn uart_udre_pending_clears_after_write() {
    let mut cpu = Cpu::new(vec![0x9598]).expect("cpu");
    cpu.io_write(0xC1, 0x28).expect("control");
    cpu.poll_uart(0x04).expect("poll");
    assert_ne!(cpu.pending & (1 << 19), 0);
    cpu.io_write(0xC6, 0x41).expect("tx");
    cpu.poll_uart(0x04).expect("poll");
    assert_eq!(cpu.pending & (1 << 19), 0);
}

#[test]
fn uart_transmits_from_cpu_cycles() {
    let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
    cpu.uart().write_control(1, 0x08);
    cpu.uart().write_tx(0x55);
    for _ in 0..120 {
        cpu.step().expect("step");
    }
    assert!(cpu.uart().tx_complete());
    assert!(cpu.uart().tx_ready());
    assert!(cpu.uart().tx_line());
}

#[test]
fn uart_frame_decodes_from_cpu_cycles() {
    let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
    cpu.uart().write_control(1, 0x08);
    cpu.uart().write_tx(0xA5);
    assert_eq!(decode_uart(&mut cpu), 0xA5);
}

fn decode_uart(cpu: &mut Cpu) -> u8 {
    let mut value = 0u8;
    for bit in 0..10 {
        step_cpu(cpu, 4);
        let level = cpu.uart().tx_line();
        if (1..=8).contains(&bit) && level {
            value |= 1 << (bit - 1);
        }
        step_cpu(cpu, 4);
    }
    value
}

fn step_cpu(cpu: &mut Cpu, count: usize) {
    for _ in 0..count {
        cpu.step().expect("step");
    }
}

#[test]
fn uart_rx_interrupt_enters_vector() {
    let mut program = vec![0u16; 40];
    program[0] = 0xCFFF;
    program[36] = ldi(17, 7);
    program[37] = 0x9518;
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    cpu.uart().receive(0x41);
    cpu.poll_uart(0x01).expect("poll");
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 37);
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn uart_rx_interrupt_is_automatic() {
    let mut program = vec![0u16; 40];
    program[0] = 0xCFFF;
    program[36] = ldi(17, 7);
    program[37] = 0x9518;
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.set_sreg(super::Sreg {
        interrupt: true,
        ..super::Sreg::default()
    });
    cpu.io_write(0xC1, 0x90).expect("uart control");
    cpu.uart().receive(0x41);
    cpu.step().expect("step");
    let trace = cpu.step().expect("step").expect("trace");
    assert_eq!(trace.pc, 37);
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn spi_transfer_advances_from_cpu_cycles() {
    let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
    cpu.spi().write_control(0x50);
    cpu.spi().set_received(0xA5);
    cpu.spi().write_data(0x5A);
    for _ in 0..40 {
        cpu.step().expect("step");
    }
    assert!(cpu.spi().spif());
    assert_eq!(cpu.spi().read_data(), 0xA5);
}

#[test]
fn adc_conversion_advances_from_cpu_cycles() {
    let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
    cpu.adc().set_input(0, 0x2AA);
    cpu.adc().write_control_a(0);
    cpu.adc().write_control_b(0xC0);
    for _ in 0..20 {
        cpu.step().expect("step");
    }
    assert!(cpu.adc().flag());
    assert_eq!(cpu.adc().read_data_low(), 0xAA);
    assert_eq!(cpu.adc().read_data_high(), 0x02);
}

#[test]
fn twi_start_completes_from_cpu_cycles() {
    let mut cpu = Cpu::new(vec![0xCFFF]).expect("cpu");
    cpu.twi().write_control(0x04);
    cpu.twi().start();
    for _ in 0..100 {
        cpu.step().expect("step");
    }
    assert!(cpu.twi().twint());
    assert_eq!(cpu.twi().status(), 0x08);
}

#[test]
fn pin_voltage_updates_digital_input() {
    let mut cpu = Cpu::new(vec![0x0000]).expect("cpu");
    let config = PinElectricalConfig::default();
    let external = ExternalPin::Thevenin {
        voltage: 5.0,
        resistance: 1000.0,
    };
    let state = cpu.set_pin_voltage(0, 5, external, &config).expect("pin");
    assert!(state.high_z);
    assert!(state.logic_level(&config));
    assert_eq!(cpu.io(0x03) & 0x20, 0x20);
}

fn pb5_output_cpu() -> Cpu {
    let program = vec![
        io_bit_op(0x9A00, 0x04, 5),
        io_bit_op(0x9A00, 0x05, 5),
        0x0000,
    ];
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.step().expect("ddr");
    cpu.step().expect("port");
    cpu
}

#[test]
fn output_contention_is_detected() {
    let cpu = pb5_output_cpu();
    let config = PinElectricalConfig::default();
    let open = cpu
        .pin_electrical(0, 5, ExternalPin::HighZ, &config)
        .expect("open");
    assert!((open.voltage - 5.0).abs() < 1e-12, "v={}", open.voltage);
    let external = ExternalPin::Thevenin {
        voltage: 0.0,
        resistance: 10.0,
    };
    let state = cpu.pin_electrical(0, 5, external, &config).expect("pin");
    assert!(state.contention);
    assert!(!state.high_z);
}
