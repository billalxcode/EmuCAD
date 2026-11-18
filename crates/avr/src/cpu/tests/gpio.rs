use super::*;

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
