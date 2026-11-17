//! Measures gate-level register-file and SRAM memory macros.

use sim_cells::catalog_digital::digital_entry;
use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, RegisterPorts, read_bits, register_file};
use std::hint::black_box;
use std::time::Instant;

const COUNT: usize = 32;
const OPS: usize = 50_000;
const SRAM_OPS: usize = 10_000;

fn main() {
    for width in [8, 16, 64] {
        run(width);
    }
    run_sram();
}

fn run(width: usize) {
    let (mut circuit, file, inputs) = build(width);
    let stats = circuit.stats();
    println!(
        "registers={COUNT} width={width} gates={} dffs={} components={}",
        stats.gates, stats.dffs, stats.components
    );
    warmup(&mut circuit, &file, &inputs);
    let start = Instant::now();
    let mut checksum = 0_u64;
    for step in 0..OPS {
        write(&mut circuit, &inputs, step);
        circuit.tick().expect("tick");
        set_bits(&mut circuit, &inputs.read_a, step);
        circuit.settle().expect("read");
        checksum ^= read_word(&circuit, file.read_a());
    }
    let seconds = start.elapsed().as_secs_f64();
    black_box(checksum);
    println!(
        "  ops={OPS} wall_s={seconds:.6} ops_per_s={:.0} ns_per_op={:.1}",
        OPS as f64 / seconds,
        seconds * 1e9 / OPS as f64
    );
}

struct Inputs {
    write_address: Vec<NetId>,
    write_data: Vec<NetId>,
    write_enable: NetId,
    read_a: Vec<NetId>,
    read_b: Vec<NetId>,
}

fn build(width: usize) -> (DigitalCircuit, sim_cells::RegisterFile, Inputs) {
    let mut builder = DigitalBuilder::new();
    let inputs = Inputs {
        write_address: nets(&mut builder, 5),
        write_data: nets(&mut builder, width),
        write_enable: builder.input("we"),
        read_a: nets(&mut builder, 5),
        read_b: nets(&mut builder, 5),
    };
    let ports = RegisterPorts {
        write_address: inputs.write_address.clone(),
        write_data: inputs.write_data.clone(),
        write_enable: inputs.write_enable,
        read_address_a: inputs.read_a.clone(),
        read_address_b: inputs.read_b.clone(),
    };
    let file = register_file(&mut builder, COUNT, width, &ports).expect("register file");
    (builder.build(), file, inputs)
}

fn nets(builder: &mut DigitalBuilder, count: usize) -> Vec<NetId> {
    (0..count).map(|_| builder.net()).collect()
}

fn warmup(circuit: &mut DigitalCircuit, file: &sim_cells::RegisterFile, inputs: &Inputs) {
    for step in 0..COUNT {
        write(circuit, inputs, step);
        circuit.tick().expect("warmup");
        set_bits(circuit, &inputs.read_a, step);
        circuit.settle().expect("warmup read");
        black_box(read_word(circuit, file.read_a()));
    }
}

fn write(circuit: &mut DigitalCircuit, inputs: &Inputs, step: usize) {
    circuit.set_net(inputs.write_enable, true);
    set_bits(circuit, &inputs.write_address, step);
    set_bits(
        circuit,
        &inputs.write_data,
        step.wrapping_mul(0x9E37_79B9_7F4A_7C15),
    );
}

fn set_bits(circuit: &mut DigitalCircuit, nets: &[NetId], value: usize) {
    for (bit, net) in nets.iter().enumerate() {
        circuit.set_net(*net, value & (1 << bit) != 0);
    }
}

fn read_word(circuit: &DigitalCircuit, nets: &[NetId]) -> u64 {
    read_bits(circuit, nets) as u64
}

fn run_sram() {
    let mut entry = digital_entry("memory.device.sram_array")
        .expect("build")
        .expect("entry");
    let stats = entry.circuit.stats();
    println!(
        "sram=128x8 gates={} dffs={} components={}",
        stats.gates, stats.dffs, stats.components
    );
    warmup_sram(&mut entry);
    let start = Instant::now();
    let mut checksum = 0_u64;
    for step in 0..SRAM_OPS {
        write_sram(&mut entry, step);
        entry.circuit.tick().expect("tick");
        read_sram(&mut entry, step);
        checksum ^= sram_value(&entry) as u64;
    }
    let seconds = start.elapsed().as_secs_f64();
    black_box(checksum);
    println!(
        "  ops={SRAM_OPS} wall_s={seconds:.6} ops_per_s={:.0} ns_per_op={:.1}",
        SRAM_OPS as f64 / seconds,
        seconds * 1e9 / SRAM_OPS as f64
    );
}

fn warmup_sram(entry: &mut sim_cells::catalog_digital::DigitalEntry) {
    for step in 0..128 {
        write_sram(entry, step);
        entry.circuit.tick().expect("warmup");
        read_sram(entry, step);
        black_box(sram_value(entry));
    }
}

fn write_sram(entry: &mut sim_cells::catalog_digital::DigitalEntry, step: usize) {
    set_sram_address(entry, step % 128);
    set_sram_data(entry, step as u8);
    entry.circuit.set_input("we", true).expect("we");
}

fn read_sram(entry: &mut sim_cells::catalog_digital::DigitalEntry, step: usize) {
    entry.circuit.set_input("we", false).expect("we");
    set_sram_address(entry, step % 128);
    entry.circuit.settle().expect("read");
}

fn set_sram_address(entry: &mut sim_cells::catalog_digital::DigitalEntry, value: usize) {
    for bit in 0..7 {
        entry
            .circuit
            .set_input(&format!("addr{bit}"), value & (1 << bit) != 0)
            .expect("addr");
    }
}

fn set_sram_data(entry: &mut sim_cells::catalog_digital::DigitalEntry, value: u8) {
    for bit in 0..8 {
        entry
            .circuit
            .set_input(&format!("din{bit}"), value & (1 << bit) != 0)
            .expect("din");
    }
}

fn sram_value(entry: &sim_cells::catalog_digital::DigitalEntry) -> u8 {
    let mut value = 0u8;
    for bit in 0..8 {
        if entry.circuit.output(&format!("dout{bit}")) == Some(true) {
            value |= 1 << bit;
        }
    }
    value
}
