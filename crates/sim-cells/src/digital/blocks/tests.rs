use super::{
    ProgramCounter, StackPointer, and_all, counter, decoder, full_adder, mux_tree, or_all,
    read_bits, register, shift_register,
};
use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};

#[test]
fn reduction_helpers_handle_empty_inputs() {
    let mut builder = DigitalBuilder::new();
    let all = and_all(&mut builder, &[]);
    let any = or_all(&mut builder, &[]);
    let mut circuit = builder.build();
    circuit.settle().expect("settle");
    assert!(circuit.value(all));
    assert!(!circuit.value(any));
}

#[test]
fn decoder_selects_one_line() {
    let mut builder = DigitalBuilder::new();
    let first = builder.input("a");
    let second = builder.input("b");
    let lines = decoder(&mut builder, &[first, second]).expect("decoder");
    for (index, line) in lines.iter().enumerate() {
        builder.output(&format!("y{index}"), *line);
    }
    let mut circuit = builder.build();
    for value in 0..4 {
        circuit.set_input("a", value & 1 != 0).expect("a");
        circuit.set_input("b", value & 2 != 0).expect("b");
        circuit.settle().expect("settle");
        for index in 0..4 {
            let expected = index == value;
            assert_eq!(circuit.output(&format!("y{index}")).expect("y"), expected);
        }
    }
}

fn mux_tree_fixture() -> DigitalCircuit {
    let mut builder = DigitalBuilder::new();
    let mut inputs = Vec::new();
    for index in 0..4 {
        inputs.push(builder.input(&format!("i{index}")));
    }
    let first = builder.input("s0");
    let second = builder.input("s1");
    let output = mux_tree(&mut builder, &inputs, &[first, second]).expect("mux");
    builder.output("y", output);
    builder.build()
}

fn set_mux_inputs(circuit: &mut DigitalCircuit, value: usize) {
    for index in 0..4 {
        circuit
            .set_input(&format!("i{index}"), index == value)
            .expect("input");
    }
    circuit.set_input("s0", value & 1 != 0).expect("s0");
    circuit.set_input("s1", value & 2 != 0).expect("s1");
}

#[test]
fn mux_tree_selects_every_input() {
    let mut circuit = mux_tree_fixture();
    for value in 0..4 {
        set_mux_inputs(&mut circuit, value);
        circuit.settle().expect("settle");
        assert!(circuit.output("y").expect("y"));
    }
}

#[test]
fn counter_counts_and_holds() {
    let mut builder = DigitalBuilder::new();
    let enable = builder.input("enable");
    let bits = counter(&mut builder, 4, enable).expect("counter");
    for (index, bit) in bits.iter().enumerate() {
        builder.output(&format!("q{index}"), *bit);
    }
    let mut circuit = builder.build();
    circuit.set_input("enable", true).expect("enable");
    for expected in 0..16 {
        assert_eq!(read_bits(&circuit, &bits), expected);
        circuit.tick().expect("tick");
    }
    assert_eq!(read_bits(&circuit, &bits), 0);
    circuit.set_input("enable", false).expect("enable");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &bits), 0);
}

fn shift_fixture() -> (DigitalCircuit, Vec<NetId>) {
    let mut builder = DigitalBuilder::new();
    let input = builder.input("din");
    let enable = builder.input("enable");
    let bits = shift_register(&mut builder, 4, input, enable).expect("shift");
    (builder.build(), bits)
}

#[test]
fn shift_register_moves_bits() {
    let (mut circuit, bits) = shift_fixture();
    circuit.set_input("enable", true).expect("enable");
    circuit.set_input("din", true).expect("din");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &bits), 0b0001);
    circuit.set_input("din", false).expect("din");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &bits), 0b0010);
    circuit.tick().expect("tick");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &bits), 0b1000);
    circuit.set_input("enable", false).expect("enable");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &bits), 0b1000);
}

fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
    (0..width)
        .map(|index| builder.input(&format!("{prefix}{index}")))
        .collect()
}

fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
    for index in 0..width {
        let bit = value & (1 << index) != 0;
        circuit
            .set_input(&format!("{prefix}{index}"), bit)
            .expect("input");
    }
}

fn pc_fixture() -> (DigitalCircuit, ProgramCounter) {
    let mut builder = DigitalBuilder::new();
    let load = builder.input("load");
    let increment = builder.input("inc");
    let target = named_inputs(&mut builder, "t", 16);
    let pc = ProgramCounter::new(&mut builder, 16, load, &target, increment).expect("pc");
    (builder.build(), pc)
}

fn sp_fixture() -> (DigitalCircuit, StackPointer) {
    let mut builder = DigitalBuilder::new();
    let load = builder.input("load");
    let increment = builder.input("inc");
    let decrement = builder.input("dec");
    let target = named_inputs(&mut builder, "t", 16);
    let sp = StackPointer::new(&mut builder, 16, load, &target, increment, decrement).expect("sp");
    (builder.build(), sp)
}

#[test]
fn program_counter_counts_and_loads() {
    let (mut circuit, pc) = pc_fixture();
    circuit.set_input("inc", true).expect("inc");
    circuit.set_input("load", false).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(pc.read(&circuit), 1);
    set_bits(&mut circuit, "t", 0x1234, 16);
    circuit.set_input("inc", false).expect("inc");
    circuit.set_input("load", true).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(pc.read(&circuit), 0x1234);
    circuit.reset().expect("reset");
    assert_eq!(pc.read(&circuit), 0);
}

#[test]
fn program_counter_holds_without_increment() {
    let (mut circuit, pc) = pc_fixture();
    circuit.set_input("inc", false).expect("inc");
    circuit.set_input("load", false).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(pc.read(&circuit), 0);
}

#[test]
fn stack_pointer_steps_and_loads() {
    let (mut circuit, sp) = sp_fixture();
    set_bits(&mut circuit, "t", 0x08FF, 16);
    circuit.set_input("load", true).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(sp.read(&circuit), 0x08FF);
    circuit.set_input("load", false).expect("load");
    circuit.set_input("inc", true).expect("inc");
    circuit.tick().expect("tick");
    assert_eq!(sp.read(&circuit), 0x0900);
    circuit.set_input("inc", false).expect("inc");
    circuit.set_input("dec", true).expect("dec");
    circuit.tick().expect("tick");
    assert_eq!(sp.read(&circuit), 0x08FF);
    circuit.set_input("dec", false).expect("dec");
    circuit.tick().expect("tick");
    assert_eq!(sp.read(&circuit), 0x08FF);
}

#[test]
fn instruction_register_loads_and_holds() {
    let mut builder = DigitalBuilder::new();
    let load = builder.input("load");
    let data = named_inputs(&mut builder, "d", 16);
    let ir = register(&mut builder, 16, &data, load).expect("ir");
    let mut circuit = builder.build();
    set_bits(&mut circuit, "d", 0xABCD, 16);
    circuit.set_input("load", true).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &ir), 0xABCD);
    set_bits(&mut circuit, "d", 0x0000, 16);
    circuit.set_input("load", false).expect("load");
    circuit.tick().expect("tick");
    assert_eq!(read_bits(&circuit, &ir), 0xABCD);
}

fn set_adder_inputs(circuit: &mut DigitalCircuit, value: u8) {
    circuit.set_input("a", value & 1 != 0).expect("a");
    circuit.set_input("b", value & 2 != 0).expect("b");
    circuit.set_input("c", value & 4 != 0).expect("c");
    circuit.settle().expect("settle");
}

#[test]
fn digital_full_adder_truth_table() {
    let mut builder = DigitalBuilder::new();
    let first = builder.input("a");
    let second = builder.input("b");
    let carry_in = builder.input("c");
    let (sum, carry_out) = full_adder(&mut builder, first, second, carry_in);
    builder.output("sum", sum);
    builder.output("carry", carry_out);
    let mut circuit = builder.build();
    for value in 0..8 {
        set_adder_inputs(&mut circuit, value);
        let total = u8::count_ones(value) as u8;
        assert_eq!(circuit.output("sum").expect("sum"), total & 1 != 0);
        assert_eq!(circuit.output("carry").expect("carry"), total >= 2);
    }
}
