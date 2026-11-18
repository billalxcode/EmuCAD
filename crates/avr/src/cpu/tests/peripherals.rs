use super::*;

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
    for _ in 0..30 {
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
