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
    let nodes = relay_nodes(&mut builder);
    add_input(&mut builder, "V1", nodes.0, nodes.4, coil_voltage);
    add_input(&mut builder, "V2", nodes.1, nodes.4, 5.0);
    add_load(&mut builder, "R1", nodes.2, nodes.4);
    add_load(&mut builder, "R2", nodes.3, nodes.4);
    add_relay(&mut builder, nodes.0, nodes.4, nodes.1, nodes.2, nodes.3);
    builder.build().expect("circuit")
}

fn relay_nodes(builder: &mut CircuitBuilder) -> (NodeId, NodeId, NodeId, NodeId, NodeId) {
    let coil = builder.node("coil").expect("node");
    let supply = builder.node("vcc").expect("node");
    let normally_open = builder.node("no").expect("node");
    let normally_closed = builder.node("nc").expect("node");
    (
        coil,
        supply,
        normally_open,
        normally_closed,
        builder.ground(),
    )
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

fn bjt_switch() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let (drive, base, collector, supply, ground) = bjt_nodes(&mut builder);
    add_pulse_source(&mut builder, drive, ground);
    add_load(&mut builder, "R1", drive, base);
    add_load(&mut builder, "R2", supply, collector);
    add_input(&mut builder, "V2", supply, ground, 5.0);
    builder
        .device(
            "Q1",
            DeviceKind::Npn,
            vec![collector, base, ground],
            params(&[("bf", 100.0)]),
        )
        .expect("Q1");
    builder.build().expect("circuit")
}

fn bjt_nodes(builder: &mut CircuitBuilder) -> (NodeId, NodeId, NodeId, NodeId, NodeId) {
    let drive = builder.node("drive").expect("node");
    let base = builder.node("base").expect("node");
    let collector = builder.node("collector").expect("node");
    let supply = builder.node("vcc").expect("node");
    (drive, base, collector, supply, builder.ground())
}

fn add_pulse_source(builder: &mut CircuitBuilder, positive: NodeId, negative: NodeId) {
    builder
        .device(
            "V1",
            DeviceKind::PulseVoltageSource,
            vec![positive, negative],
            params(&[
                ("v1", 0.0),
                ("v2", 5.0),
                ("delay", 0.0),
                ("rise", 1e-3),
                ("fall", 1e-3),
                ("width", 0.5),
                ("period", 1.0),
            ]),
        )
        .expect("V1");
}

#[test]
fn bjt_switch_transient_toggles_collector() {
    let circuit = bjt_switch();
    let config = sim_analysis::TransientConfig {
        dt: 1e-4,
        steps: 10_000,
        use_dc_op: true,
        ..sim_analysis::TransientConfig::default()
    };
    let samples = sim_analysis::transient(&circuit, &config).expect("transient");
    let collector = NodeId::new(3);
    let high = samples[2500].solution.voltage(collector).expect("high");
    let low = samples[7500].solution.voltage(collector).expect("low");
    assert!(high < 1.0, "collector high={high}");
    assert!(low > 4.0, "collector low={low}");
}

fn transformer_circuit(ratio: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let source = builder.node("source").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_transformer_source(&mut builder, source, ground);
    add_transformer(&mut builder, source, ground, output, ratio);
    add_load(&mut builder, "R1", output, ground);
    builder.build().expect("circuit")
}

fn add_transformer_source(builder: &mut CircuitBuilder, source: NodeId, ground: NodeId) {
    builder
        .device(
            "V1",
            DeviceKind::PulseVoltageSource,
            vec![source, ground],
            params(&[
                ("v1", 0.0),
                ("v2", 10.0),
                ("delay", 0.0),
                ("rise", 1e-6),
                ("fall", 1e-6),
                ("width", 1.0),
                ("period", 2.0),
            ]),
        )
        .expect("V1");
}

fn add_transformer(
    builder: &mut CircuitBuilder,
    source: NodeId,
    ground: NodeId,
    output: NodeId,
    ratio: f64,
) {
    builder
        .device(
            "T1",
            DeviceKind::Transformer,
            vec![source, ground, output, ground],
            transformer_params(ratio),
        )
        .expect("T1");
}

fn transformer_params(ratio: f64) -> BTreeMap<String, f64> {
    params(&[
        ("turns_ratio", ratio),
        ("inductance", 1.0),
        ("coupling", 0.999),
        ("rp", 0.1),
        ("rs", 0.4),
    ])
}

#[test]
fn transformer_scales_secondary_voltage() {
    let circuit = transformer_circuit(2.0);
    let config = sim_analysis::TransientConfig {
        dt: 1e-5,
        steps: 1000,
        ..sim_analysis::TransientConfig::default()
    };
    let samples = sim_analysis::transient(&circuit, &config).expect("transient");
    let output = NodeId::new(2);
    let voltage = samples[500].solution.voltage(output).expect("output");
    assert!(voltage > 15.0, "output={voltage}");
}

fn crystal_circuit(frequency: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let drive = builder.node("drive").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_crystal_source(&mut builder, drive, ground);
    add_load(&mut builder, "R1", drive, output);
    add_crystal(&mut builder, output, ground, frequency);
    builder.build().expect("circuit")
}

fn add_crystal_source(builder: &mut CircuitBuilder, drive: NodeId, ground: NodeId) {
    builder
        .device(
            "V1",
            DeviceKind::PulseVoltageSource,
            vec![drive, ground],
            params(&[
                ("v1", 0.0),
                ("v2", 1.0),
                ("delay", 0.0),
                ("rise", 1e-9),
                ("fall", 1e-9),
                ("width", 1e-6),
                ("period", 2e-6),
            ]),
        )
        .expect("V1");
}

fn add_crystal(builder: &mut CircuitBuilder, output: NodeId, ground: NodeId, frequency: f64) {
    builder
        .device(
            "X1",
            DeviceKind::Crystal,
            vec![output, ground],
            params(&[
                ("frequency", frequency),
                ("capacitance", 1e-14),
                ("resistance", 50.0),
                ("c0", 1e-12),
            ]),
        )
        .expect("X1");
}

fn crystal_crossings(frequency: f64) -> usize {
    let circuit = crystal_circuit(frequency);
    let config = sim_analysis::TransientConfig {
        dt: 1e-8,
        steps: 500,
        ..sim_analysis::TransientConfig::default()
    };
    let samples = sim_analysis::transient(&circuit, &config).expect("transient");
    let mut count = 0;
    let mut previous = samples[0].device_currents[2].unwrap_or(0.0);
    for sample in samples.iter().skip(1) {
        let value = sample.device_currents[2].unwrap_or(0.0);
        if (previous < 0.0 && value >= 0.0) || (previous > 0.0 && value <= 0.0) {
            count += 1;
        }
        previous = value;
    }
    count
}

#[test]
fn crystal_oscillates_near_its_frequency() {
    let slow = crystal_crossings(1e6);
    let fast = crystal_crossings(2e6);
    assert!(slow >= 4, "slow={slow}");
    assert!(fast > slow, "slow={slow} fast={fast}");
}

fn varistor_clamp() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let (source, output, ground) = clamp_nodes(&mut builder, "source", "out");
    add_input(&mut builder, "V1", source, ground, 10.0);
    add_load(&mut builder, "R1", source, output);
    add_varistor(&mut builder, output, ground);
    builder.build().expect("circuit")
}

fn clamp_nodes(
    builder: &mut CircuitBuilder,
    first: &str,
    second: &str,
) -> (NodeId, NodeId, NodeId) {
    let source = builder.node(first).expect("node");
    let output = builder.node(second).expect("node");
    (source, output, builder.ground())
}

fn add_varistor(builder: &mut CircuitBuilder, positive: NodeId, negative: NodeId) {
    builder
        .device(
            "RV1",
            DeviceKind::Varistor,
            vec![positive, negative],
            params(&[
                ("clamp_voltage", 5.0),
                ("clamp_current", 1.0),
                ("alpha", 4.0),
                ("leakage", 1e-9),
            ]),
        )
        .expect("RV1");
}

#[test]
fn varistor_limits_output_voltage() {
    let result = solve(&varistor_clamp());
    let output = result.solution.voltage(NodeId::new(2)).expect("output");
    assert!(output > 5.0 && output < 8.0, "output={output}");
}

fn fuse_circuit() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let (source, output, ground) = clamp_nodes(&mut builder, "source", "out");
    add_fuse_source(&mut builder, source, ground);
    add_fuse_resistor(&mut builder, source, output);
    add_fuse(&mut builder, output, ground);
    builder.build().expect("circuit")
}

fn add_fuse_source(builder: &mut CircuitBuilder, positive: NodeId, negative: NodeId) {
    builder
        .device(
            "V1",
            DeviceKind::PulseVoltageSource,
            vec![positive, negative],
            params(&[
                ("v1", 0.0),
                ("v2", 10.0),
                ("delay", 0.0),
                ("rise", 1e-9),
                ("fall", 1e-9),
                ("width", 1.0),
                ("period", 2.0),
            ]),
        )
        .expect("V1");
}

fn add_fuse_resistor(builder: &mut CircuitBuilder, positive: NodeId, negative: NodeId) {
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![positive, negative],
            params(&[("resistance", 5.0)]),
        )
        .expect("R1");
}

fn add_fuse(builder: &mut CircuitBuilder, positive: NodeId, negative: NodeId) {
    builder
        .device(
            "F1",
            DeviceKind::Fuse,
            vec![positive, negative],
            params(&[
                ("resistance", 0.1),
                ("trip_current", 1.0),
                ("trip_time", 1e-3),
                ("reset", 0.0),
            ]),
        )
        .expect("F1");
}

#[test]
fn fuse_opens_after_sustained_overcurrent() {
    let circuit = fuse_circuit();
    let config = sim_analysis::TransientConfig {
        dt: 1e-4,
        steps: 50,
        ..sim_analysis::TransientConfig::default()
    };
    let samples = sim_analysis::transient(&circuit, &config).expect("transient");
    let early = samples[3].device_currents[2].expect("early current");
    let late = samples[40].device_currents[2].expect("late current");
    assert!(early > 1.0, "early={early}");
    assert!(late.abs() < 1e-6, "late={late}");
}
