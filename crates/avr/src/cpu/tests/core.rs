use super::*;

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
fn untraced_step_updates_state() {
    let mut cpu = Cpu::new(vec![ldi(16, 5), 0x9598]).expect("cpu");
    let cycles = cpu.step_untraced().expect("step").expect("cycles");
    assert_eq!(cycles, 1);
    assert_eq!(cpu.register(16), 5);
    assert!(cpu.step_untraced().expect("step").is_some());
    assert!(cpu.step_untraced().expect("step").is_none());
}

#[test]
fn untraced_run_matches_traced_state() {
    let program = vec![ldi(16, 1), ldi(17, 2), reg_op(0x0C00, 16, 17), 0x9598];
    let mut traced = Cpu::new(program.clone()).expect("traced");
    let traces = traced.run(10).expect("run");
    let mut fast = Cpu::new(program).expect("fast");
    let executed = fast.run_untraced(10).expect("run");
    assert_eq!(executed, traces.len() as u64);
    assert_eq!(fast.register(16), traced.register(16));
    assert_eq!(fast.cycles(), traced.cycles());
}
