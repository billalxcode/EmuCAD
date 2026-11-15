//! Measures AVR instruction throughput in a fixed loop.

use sim_avr::Cpu;
use std::time::Instant;

const ALU: [u16; 6] = [0xE001, 0xE012, 0x0F01, 0x1B01, 0x2F01, 0xCFFC];
const MIXED: [u16; 13] = [
    0xE505, 0xE01F, 0x2301, 0x2B01, 0x2701, 0x9506, 0x9505, 0x9507, 0x9502, 0x9503, 0x950A, 0x9F01,
    0xCFF5,
];
const MEMORY: [u16; 7] = [0xE0A0, 0xE0B2, 0xE505, 0x930C, 0x911C, 0x0F01, 0xCFFC];
const TIMER: [u16; 7] = [0xE001, 0xBD05, 0xE012, 0x0F01, 0x1B01, 0x2F01, 0xCFFC];
const TIMER64: [u16; 7] = [0xE003, 0xBD05, 0xE012, 0x0F01, 0x1B01, 0x2F01, 0xCFFC];
const GPIO: [u16; 9] = [
    0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
];
const CYCLES_PER_SECOND: f64 = 16_000_000.0;
const WARMUP: usize = 10_000;

struct Measurement {
    instructions: usize,
    cycles: u64,
    seconds: f64,
}

fn main() {
    let steps = parse_arg(1, 1_000_000);
    let mode = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "both".to_string());
    print_stats();
    for (name, program) in [
        ("alu", &ALU[..]),
        ("mixed", &MIXED[..]),
        ("memory", &MEMORY[..]),
        ("timer", &TIMER[..]),
        ("timer64", &TIMER64[..]),
        ("gpio", &GPIO[..]),
    ] {
        run_program(name, program, steps, &mode);
    }
}

fn parse_arg(position: usize, default: usize) -> usize {
    std::env::args()
        .nth(position)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn print_stats() {
    let stats = Cpu::new(ALU.to_vec()).expect("cpu").circuit_stats();
    println!(
        "gates={} components={} dffs={} largest_component={} cyclic_components={}",
        stats.gates, stats.components, stats.dffs, stats.largest_component, stats.cyclic_components
    );
}

fn run_program(name: &str, program: &[u16], steps: usize, mode: &str) {
    if mode != "untraced" {
        report(name, "traced", measure(program, steps, true));
    }
    if mode != "traced" {
        report(name, "untraced", measure(program, steps, false));
    }
}

fn measure(program: &[u16], steps: usize, traced: bool) -> Measurement {
    let mut cpu = Cpu::new(program.to_vec()).expect("cpu");
    for _ in 0..WARMUP {
        step(&mut cpu, traced).expect("warmup");
    }
    let before = cpu.cycles();
    let start = Instant::now();
    let mut instructions = 0;
    for _ in 0..steps {
        if step(&mut cpu, traced).expect("step").is_none() {
            break;
        }
        instructions += 1;
    }
    Measurement {
        instructions,
        cycles: cpu.cycles() - before,
        seconds: start.elapsed().as_secs_f64(),
    }
}

fn step(cpu: &mut Cpu, traced: bool) -> Result<Option<u8>, sim_core::SimError> {
    if traced {
        return cpu.step().map(|trace| trace.map(|value| value.cycles));
    }
    cpu.step_untraced()
}

fn report(program: &str, method: &str, measurement: Measurement) {
    let seconds = measurement.seconds;
    if measurement.instructions == 0 || seconds <= 0.0 {
        println!("{program}/{method}: no instructions executed");
        return;
    }
    let instructions = measurement.instructions as f64 / seconds;
    let cycles = measurement.cycles as f64 / seconds;
    let nanoseconds = seconds * 1.0e9 / measurement.instructions as f64;
    let realtime = cycles / CYCLES_PER_SECOND;
    println!(
        "{program:5} {method:9} instructions={} cycles={} wall_s={seconds:.6} instr_per_s={instructions:.0} cycles_per_s={cycles:.0} ns_per_instr={nanoseconds:.1} realtime={realtime:.3}x",
        measurement.instructions, measurement.cycles
    );
}
