use super::*;

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
