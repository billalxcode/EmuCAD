use super::hex::hex_words;
use super::{Event, Sample, SampleState, SimHandle, SimMachine, Waveform};

fn blink_program() -> Vec<u16> {
    vec![
        0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
    ]
}

#[test]
fn uart_tx_bytes_are_collected() {
    let mut machine = SimMachine::new(blink_program()).expect("machine");
    machine.cpu.uart().write_control(1, 0x08);
    machine.cpu.uart().write_tx(b'A');
    machine.cpu.uart().tick(200);
    machine.update_uart();
    assert_eq!(machine.take_uart(), vec![b'A']);
    assert!(machine.take_uart().is_empty());
}

#[test]
fn uart_rx_bytes_are_delivered() {
    let mut machine = SimMachine::new(blink_program()).expect("machine");
    machine.queue_uart(b"Hi");
    machine.pump_uart_rx();
    assert!(machine.cpu.uart().rx_ready());
    assert_eq!(machine.cpu.uart().read_rx(), b'H');
    machine.pump_uart_rx();
    assert_eq!(machine.cpu.uart().read_rx(), b'i');
}

#[test]
fn worker_loads_program_and_reports_status() {
    let handle = SimHandle::spawn();
    handle.load(blink_program()).expect("load");
    let status = handle.wait_status().expect("status");
    assert!(status.loaded);
    assert!(!status.running);
    assert_eq!(status.cycles, 0);
}

#[test]
fn worker_runs_and_pauses() {
    let handle = SimHandle::spawn();
    handle.load(blink_program()).expect("load");
    handle.wait_status().expect("load status");
    handle.run().expect("run");
    let running = handle
        .wait_until(|status| status.running && status.cycles > 0)
        .expect("running status");
    assert!(running.cycles > 0);
    handle.pause().expect("pause");
    let paused = handle
        .wait_until(|status| !status.running)
        .expect("paused status");
    assert!(!paused.running);
}

#[test]
fn blink_program_toggles_board_led() {
    let mut machine = SimMachine::new(blink_program()).expect("machine");
    let mut seen = false;
    let mut high_voltage = false;
    for _ in 0..5000 {
        if !machine.step().expect("step") {
            break;
        }
        seen |= machine.led_on;
        high_voltage |= machine.led_on && machine.pb5_millivolts > 0;
    }
    assert!(seen);
    assert!(high_voltage);
}

#[test]
fn sampling_respects_interval() {
    let mut machine = SimMachine::new(blink_program()).expect("machine");
    let mut last = SampleState::default();
    let (events, received) = std::sync::mpsc::channel();
    for _ in 0..10 {
        machine.step().expect("step");
        super::sample_step(&machine, &mut last, 10, &events);
    }
    let cycles: Vec<_> = received
        .try_iter()
        .filter_map(|event| match event {
            Event::Sample(sample) => Some(sample.cycle),
            _ => None,
        })
        .collect();
    assert!(cycles.contains(&10));
}

#[test]
fn reset_clears_cycles_and_led() {
    let mut machine = SimMachine::new(blink_program()).expect("machine");
    for _ in 0..200 {
        machine.step().expect("step");
    }
    machine.reset().expect("reset");
    assert_eq!(machine.cycles(), 0);
    assert!(!machine.led_on);
}

#[test]
fn waveform_span_extends_to_current_cycle() {
    let mut waveform = Waveform::default();
    waveform.push(sample(10, true));
    waveform.push(sample(40, false));
    assert_eq!(waveform.span(100), (10, 100));
    assert_eq!(waveform.span(20), (10, 40));
    assert_eq!(waveform.state_at(0), (false, false));
    assert_eq!(waveform.state_at(10), (true, true));
    assert_eq!(waveform.state_at(40), (false, false));
}

fn sample(cycle: u64, high: bool) -> Sample {
    Sample {
        cycle,
        pb5_high: high,
        pb5_millivolts: if high { 5000 } else { 0 },
        led_on: high,
        led_microamps: if high { 14_000 } else { 0 },
    }
}

#[test]
fn waveform_is_bounded() {
    let mut waveform = Waveform::default();
    for cycle in 0..super::WAVE_LIMIT as u64 + 5 {
        waveform.push(Sample {
            cycle,
            pb5_high: cycle % 2 == 0,
            pb5_millivolts: if cycle % 2 == 0 { 5000 } else { 0 },
            led_on: cycle % 2 == 0,
            led_microamps: if cycle % 2 == 0 { 14_000 } else { 0 },
        });
    }
    assert_eq!(waveform.samples().len(), super::WAVE_LIMIT);
    assert_eq!(waveform.span(9999), (5, 9999));
}

#[test]
fn hex_text_becomes_words() {
    let text = sim_board::to_intel_hex(&[0x01, 0x02, 0x03, 0x04]);
    let words = hex_words(&text).expect("hex");
    assert_eq!(&words[..2], &[0x0201, 0x0403]);
    assert_eq!(words.len(), super::FLASH_BYTES / 2);
}

#[test]
#[ignore = "wall-clock release test"]
fn worker_realtime_pacing_matches_clock() {
    use std::time::{Duration, Instant};

    let handle = SimHandle::spawn();
    handle.load(blink_program()).expect("load");
    handle.wait_status().expect("load status");
    let start = Instant::now();
    handle.run().expect("run");
    let status = handle
        .wait_until(|status| status.running && status.cycles >= 16_000_000)
        .expect("status");
    let expected = Duration::from_secs_f64(status.cycles as f64 / 16_000_000.0);
    let error = (start.elapsed().as_secs_f64() - expected.as_secs_f64()) / expected.as_secs_f64();
    assert!(error.abs() < 0.01, "realtime error {error:.4}");
    handle.pause().expect("pause");
}
