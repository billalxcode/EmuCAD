use sim_analysis::{DcConfig, dc_operating_point};
use sim_core::{Circuit, CircuitBuilder, DeviceKind, NodeId};
use std::collections::BTreeMap;

fn params(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_string(), *value))
        .collect()
}

fn opamp_follower(input: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let signal = builder.node("in").expect("node");
    let output = builder.node("out").expect("node");
    let supply = builder.node("vcc").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", signal, ground, input);
    add_input(&mut builder, "V2", supply, ground, 5.0);
    builder
        .device(
            "U1",
            DeviceKind::OpAmp,
            vec![signal, output, supply, ground, output],
            params(&[("gain", 1e5)]),
        )
        .expect("U1");
    builder.build().expect("circuit")
}

fn and_gate(first: f64, second: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let input_a = builder.node("a").expect("node");
    let input_b = builder.node("b").expect("node");
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", input_a, ground, first);
    add_input(&mut builder, "V2", input_b, ground, second);
    add_input(&mut builder, "V3", supply, ground, 5.0);
    add_load(&mut builder, "R1", output, ground);
    add_and_gate(&mut builder, input_a, input_b, supply, ground, output);
    builder.build().expect("circuit")
}

fn add_and_gate(
    builder: &mut CircuitBuilder,
    input_a: NodeId,
    input_b: NodeId,
    supply: NodeId,
    ground: NodeId,
    output: NodeId,
) {
    builder
        .device(
            "U1",
            DeviceKind::LogicGate,
            vec![input_a, input_b, supply, ground, output],
            params(&[("gate", 0.0), ("vth", 2.5), ("width", 0.1)]),
        )
        .expect("U1");
}

fn add_input(
    builder: &mut CircuitBuilder,
    name: &str,
    positive: NodeId,
    negative: NodeId,
    voltage: f64,
) {
    builder
        .device(
            name,
            DeviceKind::VoltageSource,
            vec![positive, negative],
            params(&[("voltage", voltage)]),
        )
        .expect("source");
}

fn relay_driver(coil_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let coil = builder.node("coil").expect("node");
    let supply = builder.node("vcc").expect("node");
    let normally_open = builder.node("no").expect("node");
    let normally_closed = builder.node("nc").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", coil, ground, coil_voltage);
    add_input(&mut builder, "V2", supply, ground, 5.0);
    add_load(&mut builder, "R1", normally_open, ground);
    add_load(&mut builder, "R2", normally_closed, ground);
    add_relay(
        &mut builder,
        coil,
        ground,
        supply,
        normally_open,
        normally_closed,
    );
    builder.build().expect("circuit")
}

fn add_relay(
    builder: &mut CircuitBuilder,
    coil: NodeId,
    ground: NodeId,
    supply: NodeId,
    normally_open: NodeId,
    normally_closed: NodeId,
) {
    builder
        .device(
            "K1",
            DeviceKind::Relay,
            vec![coil, ground, supply, normally_open, normally_closed],
            params(&[("rcoil", 100.0), ("ron", 0.1), ("roff", 1e9), ("vth", 2.5)]),
        )
        .expect("K1");
}

fn add_load(builder: &mut CircuitBuilder, name: &str, positive: NodeId, negative: NodeId) {
    builder
        .device(
            name,
            DeviceKind::Resistor,
            vec![positive, negative],
            params(&[("resistance", 1000.0)]),
        )
        .expect("load");
}

fn solve(circuit: &Circuit) -> sim_analysis::DcResult {
    dc_operating_point(circuit, &DcConfig::default()).expect("dc")
}

#[test]
fn opamp_follower_tracks_input() {
    let result = solve(&opamp_follower(2.0));
    let output = result.solution.voltage(NodeId::new(2)).expect("output");
    assert!((output - 2.0).abs() < 0.05, "output={output}");
}

#[test]
fn and_gate_requires_both_inputs() {
    let high = solve(&and_gate(5.0, 5.0));
    let low = solve(&and_gate(5.0, 0.0));
    let output = NodeId::new(4);
    assert!(high.solution.voltage(output).expect("high") > 4.5);
    assert!(low.solution.voltage(output).expect("low") < 0.5);
}

#[test]
fn relay_contacts_follow_coil_state() {
    let energized = solve(&relay_driver(5.0));
    let released = solve(&relay_driver(0.0));
    assert_high(&energized, NodeId::new(3));
    assert_low(&released, NodeId::new(3));
    assert_high(&released, NodeId::new(4));
}

fn assert_high(result: &sim_analysis::DcResult, node: NodeId) {
    let value = result.solution.voltage(node).expect("voltage");
    assert!(value > 4.5, "voltage={value}");
}

fn assert_low(result: &sim_analysis::DcResult, node: NodeId) {
    let value = result.solution.voltage(node).expect("voltage");
    assert!(value < 0.5, "voltage={value}");
}
