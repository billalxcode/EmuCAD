use super::{ClockDomain, DigitalBuilder, DigitalCircuit};
use kernel::SimError;

fn combinational_circuit() -> DigitalCircuit {
    let mut builder = DigitalBuilder::new();
    let a = builder.input("a");
    let b = builder.input("b");
    let not = builder.not(a);
    let and = builder.and(a, b);
    let or = builder.or(a, b);
    let xor = builder.xor(a, b);
    let nand = builder.nand(a, b);
    let nor = builder.nor(a, b);
    builder.output("not", not);
    builder.output("and", and);
    builder.output("or", or);
    builder.output("xor", xor);
    builder.output("nand", nand);
    builder.output("nor", nor);
    builder.build()
}

fn combinational(first: bool, second: bool) -> (bool, bool, bool, bool, bool, bool) {
    let mut circuit = combinational_circuit();
    circuit.set_input("a", first).expect("a");
    circuit.set_input("b", second).expect("b");
    circuit.settle().expect("settle");
    combinational_outputs(&circuit)
}

fn combinational_outputs(circuit: &DigitalCircuit) -> (bool, bool, bool, bool, bool, bool) {
    (
        circuit.output("not").expect("not"),
        circuit.output("and").expect("and"),
        circuit.output("or").expect("or"),
        circuit.output("xor").expect("xor"),
        circuit.output("nand").expect("nand"),
        circuit.output("nor").expect("nor"),
    )
}

#[test]
fn combinational_gates_follow_truth_tables() {
    assert_eq!(
        combinational(false, false),
        (true, false, false, false, true, true)
    );
    assert_eq!(
        combinational(false, true),
        (true, false, true, true, true, false)
    );
    assert_eq!(
        combinational(true, false),
        (false, false, true, true, true, false)
    );
    assert_eq!(
        combinational(true, true),
        (false, true, true, false, false, false)
    );
}

#[test]
fn buffer_follows_input() {
    let mut builder = DigitalBuilder::new();
    let input = builder.input("a");
    let output = builder.net();
    builder.buf_into(input, output);
    builder.output("y", output);
    let mut circuit = builder.build();
    circuit.set_input("a", true).expect("a");
    circuit.settle().expect("settle");
    assert!(circuit.output("y").expect("y"));
    circuit.set_input("a", false).expect("a");
    circuit.settle().expect("settle");
    assert!(!circuit.output("y").expect("y"));
}

#[test]
fn mux_selects_inputs() {
    let mut builder = DigitalBuilder::new();
    let a = builder.input("a");
    let b = builder.input("b");
    let select = builder.input("sel");
    let output = builder.mux(a, b, select);
    builder.output("y", output);
    let mut circuit = builder.build();
    circuit.set_input("a", true).expect("a");
    circuit.set_input("b", false).expect("b");
    circuit.set_input("sel", false).expect("sel");
    circuit.settle().expect("settle");
    assert!(circuit.output("y").expect("y"));
    circuit.set_input("sel", true).expect("sel");
    circuit.settle().expect("settle");
    assert!(!circuit.output("y").expect("y"));
}

#[test]
fn dff_captures_on_tick_and_holds() {
    let mut builder = DigitalBuilder::new();
    let data = builder.input("d");
    let output = builder.dff(data);
    builder.output("q", output);
    let mut circuit = builder.build();
    circuit.set_input("d", true).expect("d");
    circuit.tick().expect("tick");
    assert!(circuit.output("q").expect("q"));
    circuit.set_input("d", false).expect("d");
    assert!(circuit.output("q").expect("q"));
    circuit.tick().expect("tick");
    assert!(!circuit.output("q").expect("q"));
}

#[test]
fn reset_clears_flip_flops() {
    let mut builder = DigitalBuilder::new();
    let data = builder.input("d");
    let output = builder.dff(data);
    builder.output("q", output);
    let mut circuit = builder.build();
    circuit.set_input("d", true).expect("d");
    circuit.tick().expect("tick");
    circuit.reset().expect("reset");
    assert!(!circuit.output("q").expect("q"));
}

#[test]
fn independent_components_settle_after_each_change() {
    let mut builder = DigitalBuilder::new();
    let a = builder.input("a");
    let b = builder.input("b");
    let first = builder.not(a);
    let second = builder.not(b);
    builder.output("first", first);
    builder.output("second", second);
    let mut circuit = builder.build();
    circuit.set_input("a", true).expect("a");
    circuit.settle().expect("settle");
    assert!(!circuit.output("first").expect("first"));
    assert!(circuit.output("second").expect("second"));
    circuit.set_input("b", true).expect("b");
    circuit.settle().expect("settle");
    assert!(!circuit.output("second").expect("second"));
    circuit.set_input("a", false).expect("a");
    circuit.settle().expect("settle");
    assert!(circuit.output("first").expect("first"));
}

#[test]
fn unknown_input_is_rejected() {
    let builder = DigitalBuilder::new();
    let mut circuit = builder.build();
    let error = circuit.set_input("missing", true).expect_err("error");
    assert!(matches!(error, SimError::InvalidNetlist(_)));
}

#[test]
fn combinational_loop_reports_failure() {
    let mut builder = DigitalBuilder::new();
    let net = builder.net();
    builder.not_into(net, net);
    builder.output("y", net);
    let mut circuit = builder.build();
    let error = circuit.settle().expect_err("loop");
    assert!(matches!(error, SimError::ConvergenceFailure { .. }));
}

#[test]
fn stats_report_gate_and_component_counts() {
    let mut builder = DigitalBuilder::new();
    let input = builder.input("a");
    let output = builder.not(input);
    builder.output("y", output);
    let circuit = builder.build();
    let stats = circuit.stats();
    assert_eq!(stats.gates, 1);
    assert_eq!(stats.components, 1);
    assert_eq!(stats.dffs, 0);
    assert_eq!(stats.largest_component, 1);
    assert_eq!(stats.cyclic_components, 0);
}

#[test]
fn tick_domain_updates_only_matching_flip_flops() {
    let mut builder = DigitalBuilder::new();
    let data = builder.input("d");
    let first = builder.dff_in(ClockDomain::new(0), data);
    let second = builder.dff_in(ClockDomain::new(1), data);
    builder.output("q0", first);
    builder.output("q1", second);
    let mut circuit = builder.build();
    circuit.set_input("d", true).expect("d");
    circuit.tick_domain(ClockDomain::new(0)).expect("tick0");
    assert_eq!(circuit.output("q0"), Some(true));
    assert_eq!(circuit.output("q1"), Some(false));
    circuit.tick_domain(ClockDomain::new(1)).expect("tick1");
    assert_eq!(circuit.output("q1"), Some(true));
}
