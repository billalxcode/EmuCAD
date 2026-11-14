use super::*;

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
