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

fn igbt_params(body_is: f64) -> BTreeMap<String, f64> {
    params(&[
        ("vth", 5.0),
        ("kp", 1.0),
        ("lambda", 0.01),
        ("vce_sat", 0.5),
        ("is", body_is),
        ("n", 1.0),
        ("rs", 0.01),
    ])
}

fn add_igbt(
    builder: &mut CircuitBuilder,
    collector: NodeId,
    gate: NodeId,
    emitter: NodeId,
    body_is: f64,
) {
    builder
        .device(
            "Q1",
            DeviceKind::Igbt,
            vec![collector, gate, emitter],
            igbt_params(body_is),
        )
        .expect("Q1");
}

fn igbt_switch(gate_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let collector = builder.node("collector").expect("node");
    let gate = builder.node("gate").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 10.0);
    add_input(&mut builder, "V2", gate, ground, gate_voltage);
    add_load(&mut builder, "R1", supply, collector);
    add_igbt(&mut builder, collector, gate, ground, 1e-12);
    builder.build().expect("circuit")
}

#[test]
fn igbt_switches_with_gate_voltage() {
    let on = solve(&igbt_switch(10.0));
    let off = solve(&igbt_switch(0.0));
    assert_low(&on, NodeId::new(2));
    assert_high(&off, NodeId::new(2));
}

fn igbt_body_diode() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let emitter = builder.node("emitter").expect("node");
    let collector = builder.node("collector").expect("node");
    let gate = builder.node("gate").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", emitter, ground, 5.0);
    add_input(&mut builder, "V2", gate, ground, 0.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![collector, ground],
            params(&[("resistance", 100.0)]),
        )
        .expect("R1");
    add_igbt(&mut builder, collector, gate, emitter, 1e-6);
    builder.build().expect("circuit")
}

#[test]
fn igbt_body_diode_conducts_in_reverse() {
    let result = solve(&igbt_body_diode());
    let collector = result.solution.voltage(NodeId::new(2)).expect("collector");
    assert!(collector > 3.5, "collector={collector}");
}

fn ujt_circuit(emitter_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let base2 = builder.node("b2").expect("node");
    let source = builder.node("e_source").expect("node");
    let emitter = builder.node("emitter").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", base2, ground, 10.0);
    add_input(&mut builder, "V2", source, ground, emitter_voltage);
    add_load(&mut builder, "R1", source, emitter);
    add_ujt(&mut builder, ground, base2, emitter);
    builder.build().expect("circuit")
}

fn add_ujt(builder: &mut CircuitBuilder, base1: NodeId, base2: NodeId, emitter: NodeId) {
    builder
        .device(
            "Q1",
            DeviceKind::Ujt,
            vec![base1, base2, emitter],
            params(&[
                ("rb", 5000.0),
                ("eta", 0.6),
                ("vd", 0.7),
                ("ron", 10.0),
                ("iv", 1e-3),
                ("leakage", 1e-9),
            ]),
        )
        .expect("Q1");
}

#[test]
fn ujt_triggers_above_standoff_voltage() {
    let on = solve(&ujt_circuit(8.0));
    let off = solve(&ujt_circuit(5.0));
    let on_emitter = on.solution.voltage(NodeId::new(3)).expect("on emitter");
    let off_emitter = off.solution.voltage(NodeId::new(3)).expect("off emitter");
    assert!(on_emitter < 1.5, "on={on_emitter}");
    assert!(off_emitter > 4.5, "off={off_emitter}");
}

fn put_circuit(anode_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let gate = builder.node("gate").expect("node");
    let source = builder.node("a_source").expect("node");
    let anode = builder.node("anode").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", gate, ground, 5.0);
    add_input(&mut builder, "V2", source, ground, anode_voltage);
    add_load(&mut builder, "R1", source, anode);
    add_put(&mut builder, anode, gate, ground);
    builder.build().expect("circuit")
}

fn add_put(builder: &mut CircuitBuilder, anode: NodeId, gate: NodeId, cathode: NodeId) {
    builder
        .device(
            "Q1",
            DeviceKind::Put,
            vec![anode, gate, cathode],
            params(&[
                ("rg", 1e6),
                ("vd", 0.7),
                ("ron", 10.0),
                ("iv", 1e-3),
                ("leakage", 1e-9),
            ]),
        )
        .expect("Q1");
}

#[test]
fn put_triggers_above_gate_voltage() {
    let on = solve(&put_circuit(8.0));
    let off = solve(&put_circuit(4.0));
    let on_anode = on.solution.voltage(NodeId::new(3)).expect("on anode");
    let off_anode = off.solution.voltage(NodeId::new(3)).expect("off anode");
    assert!(on_anode < 1.5, "on={on_anode}");
    assert!(off_anode > 3.5, "off={off_anode}");
}

const SCR_VALUES: &[(&str, f64)] = &[
    ("vgt", 2.0),
    ("holding_current", 0.01),
    ("ron", 1.0),
    ("vd", 0.7),
    ("leakage", 1e-9),
];

const TRIAC_VALUES: &[(&str, f64)] = &[
    ("vgt", 5.0),
    ("holding_current", 0.01),
    ("ron", 1.0),
    ("vd", 1.0),
    ("leakage", 1e-9),
];

const DIAC_VALUES: &[(&str, f64)] = &[
    ("vbo", 30.0),
    ("holding_current", 0.01),
    ("ron", 10.0),
    ("vd", 1.0),
    ("leakage", 1e-9),
];

const SCS_VALUES: &[(&str, f64)] = &[
    ("vgt", 5.0),
    ("vgt_off", 1.0),
    ("holding_current", 0.01),
    ("ron", 1.0),
    ("vd", 0.7),
    ("leakage", 1e-9),
];

fn add_thyristor(
    builder: &mut CircuitBuilder,
    kind: DeviceKind,
    pins: Vec<NodeId>,
    values: &[(&str, f64)],
) {
    builder
        .device("Q1", kind, pins, params(values))
        .expect("Q1");
}

fn scr_circuit(gate_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let anode = builder.node("anode").expect("node");
    let gate = builder.node("gate").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 10.0);
    add_input(&mut builder, "V2", gate, ground, gate_voltage);
    add_load(&mut builder, "R1", supply, anode);
    add_thyristor(
        &mut builder,
        DeviceKind::Scr,
        vec![anode, gate, ground],
        SCR_VALUES,
    );
    builder.build().expect("circuit")
}

#[test]
fn scr_latches_on_gate_drive() {
    let on = solve(&scr_circuit(3.0));
    let off = solve(&scr_circuit(0.0));
    let on_anode = on.solution.voltage(NodeId::new(2)).expect("on anode");
    let off_anode = off.solution.voltage(NodeId::new(2)).expect("off anode");
    assert!(on_anode < 1.5, "on={on_anode}");
    assert!(off_anode > 9.0, "off={off_anode}");
}

fn triac_circuit(gate_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let mt2 = builder.node("mt2").expect("node");
    let gate = builder.node("gate").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 10.0);
    add_input(&mut builder, "V2", gate, ground, gate_voltage);
    add_load(&mut builder, "R1", supply, mt2);
    add_thyristor(
        &mut builder,
        DeviceKind::Triac,
        vec![mt2, gate, ground],
        TRIAC_VALUES,
    );
    builder.build().expect("circuit")
}

#[test]
fn triac_latches_on_either_gate_polarity() {
    let positive = solve(&triac_circuit(6.0));
    let negative = solve(&triac_circuit(-6.0));
    let off = solve(&triac_circuit(0.0));
    let positive_mt2 = positive.solution.voltage(NodeId::new(2)).expect("positive");
    let negative_mt2 = negative.solution.voltage(NodeId::new(2)).expect("negative");
    let off_mt2 = off.solution.voltage(NodeId::new(2)).expect("off");
    assert!(positive_mt2 < 1.5, "positive={positive_mt2}");
    assert!(negative_mt2 < 1.5, "negative={negative_mt2}");
    assert!(off_mt2 > 9.0, "off={off_mt2}");
}

fn diac_circuit(supply_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let first = builder.node("a1").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, supply_voltage);
    add_load(&mut builder, "R1", supply, first);
    add_thyristor(
        &mut builder,
        DeviceKind::Diac,
        vec![first, ground],
        DIAC_VALUES,
    );
    builder.build().expect("circuit")
}

#[test]
fn diac_breaks_over_above_threshold() {
    let on = solve(&diac_circuit(40.0));
    let off = solve(&diac_circuit(20.0));
    let on_a1 = on.solution.voltage(NodeId::new(2)).expect("on");
    let off_a1 = off.solution.voltage(NodeId::new(2)).expect("off");
    assert!(on_a1 < 10.0, "on={on_a1}");
    assert!(off_a1 > 15.0, "off={off_a1}");
}

fn gto_circuit(gate_voltage: f64, turn_off: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let anode = builder.node("anode").expect("node");
    let gate = builder.node("gate").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 10.0);
    add_input(&mut builder, "V2", gate, ground, gate_voltage);
    add_load(&mut builder, "R1", supply, anode);
    add_gto(&mut builder, vec![anode, gate, ground], turn_off);
    builder.build().expect("circuit")
}

fn add_gto(builder: &mut CircuitBuilder, pins: Vec<NodeId>, turn_off: f64) {
    let values = [
        ("vgt", 5.0),
        ("vgt_off", turn_off),
        ("holding_current", 0.01),
        ("ron", 1.0),
        ("vd", 0.7),
        ("leakage", 1e-9),
    ];
    add_thyristor(builder, DeviceKind::Gto, pins, &values);
}

#[test]
fn gto_and_mct_switch_with_gate() {
    let gto_on = solve(&gto_circuit(6.0, -5.0));
    let mct_on = solve(&gto_circuit(6.0, 0.0));
    let mct_off = solve(&gto_circuit(0.0, 0.0));
    let gto_anode = gto_on.solution.voltage(NodeId::new(2)).expect("gto");
    let mct_anode = mct_on.solution.voltage(NodeId::new(2)).expect("mct");
    let mct_off_anode = mct_off.solution.voltage(NodeId::new(2)).expect("mct off");
    assert!(gto_anode < 1.5, "gto={gto_anode}");
    assert!(mct_anode < 1.5, "mct={mct_anode}");
    assert!(mct_off_anode > 9.0, "mct_off={mct_off_anode}");
}

fn scs_circuit(gate_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let anode = builder.node("anode").expect("node");
    let anode_gate = builder.node("ag").expect("node");
    let cathode_gate = builder.node("cg").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 10.0);
    add_input(&mut builder, "V2", cathode_gate, ground, gate_voltage);
    add_input(&mut builder, "V3", anode_gate, ground, 0.0);
    add_load(&mut builder, "R1", supply, anode);
    add_thyristor(
        &mut builder,
        DeviceKind::Scs,
        vec![anode, anode_gate, cathode_gate, ground],
        SCS_VALUES,
    );
    builder.build().expect("circuit")
}

#[test]
fn scs_latches_on_cathode_gate() {
    let on = solve(&scs_circuit(6.0));
    let off = solve(&scs_circuit(0.0));
    let on_anode = on.solution.voltage(NodeId::new(2)).expect("on");
    let off_anode = off.solution.voltage(NodeId::new(2)).expect("off");
    assert!(on_anode < 1.5, "on={on_anode}");
    assert!(off_anode > 9.0, "off={off_anode}");
}

fn bridge_circuit(input_voltage: f64, forward_voltage: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let ac1 = builder.node("ac1").expect("node");
    let positive = builder.node("pos").expect("node");
    let negative = builder.node("neg").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", ac1, ground, input_voltage);
    add_load(&mut builder, "R1", positive, negative);
    add_bridge(
        &mut builder,
        ac1,
        ground,
        positive,
        negative,
        forward_voltage,
    );
    builder.build().expect("circuit")
}

fn add_bridge(
    builder: &mut CircuitBuilder,
    ac1: NodeId,
    ac2: NodeId,
    positive: NodeId,
    negative: NodeId,
    forward_voltage: f64,
) {
    let pins = vec![ac1, ac2, positive, negative];
    builder
        .device(
            "B1",
            DeviceKind::DiodeBridge,
            pins,
            bridge_params(forward_voltage),
        )
        .expect("B1");
}

fn bridge_params(forward_voltage: f64) -> BTreeMap<String, f64> {
    params(&[
        ("forward_voltage", forward_voltage),
        ("n", 1.0),
        ("rs", 0.01),
    ])
}

fn bridge_output(circuit: &Circuit) -> f64 {
    let result = solve(circuit);
    let positive = result.solution.voltage(NodeId::new(2)).expect("positive");
    let negative = result.solution.voltage(NodeId::new(3)).expect("negative");
    positive - negative
}

#[test]
fn bridge_rectifies_both_input_polarities() {
    let positive = bridge_output(&bridge_circuit(10.0, 0.7));
    let negative = bridge_output(&bridge_circuit(-10.0, 0.7));
    assert!(positive > 7.5, "positive={positive}");
    assert!(negative > 7.5, "negative={negative}");
}

#[test]
fn schottky_bridge_has_lower_drop() {
    let silicon = bridge_output(&bridge_circuit(10.0, 0.7));
    let schottky = bridge_output(&bridge_circuit(10.0, 0.3));
    assert!(
        schottky > silicon + 0.3,
        "silicon={silicon} schottky={schottky}"
    );
}

fn dual_diode_circuit() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let source1 = builder.node("s1").expect("node");
    let anode1 = builder.node("a1").expect("node");
    let source2 = builder.node("s2").expect("node");
    let anode2 = builder.node("a2").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", source1, ground, 5.0);
    add_input(&mut builder, "V2", source2, ground, 5.0);
    add_load(&mut builder, "R1", source1, anode1);
    add_load(&mut builder, "R2", source2, anode2);
    add_dual_diode(&mut builder, anode1, ground, anode2, ground);
    builder.build().expect("circuit")
}

fn add_dual_diode(builder: &mut CircuitBuilder, a1: NodeId, k1: NodeId, a2: NodeId, k2: NodeId) {
    builder
        .device(
            "D1",
            DeviceKind::DualDiode,
            vec![a1, k1, a2, k2],
            params(&[("forward_voltage", 0.7), ("n", 1.0), ("rs", 0.01)]),
        )
        .expect("D1");
}

#[test]
fn dual_diode_conducts_on_both_paths() {
    let result = solve(&dual_diode_circuit());
    let first = result.solution.voltage(NodeId::new(2)).expect("first");
    let second = result.solution.voltage(NodeId::new(4)).expect("second");
    assert!(first > 0.3 && first < 1.0, "first={first}");
    assert!(second > 0.3 && second < 1.0, "second={second}");
}

fn optocoupler_circuit(input_voltage: f64, ctr: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("input").expect("node");
    let anode = builder.node("anode").expect("node");
    let supply = builder.node("vcc").expect("node");
    let collector = builder.node("collector").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", input, ground, input_voltage);
    add_input(&mut builder, "V2", supply, ground, 5.0);
    add_load(&mut builder, "R1", input, anode);
    add_load(&mut builder, "R2", supply, collector);
    builder
        .device(
            "U1",
            DeviceKind::Optocoupler,
            vec![anode, ground, collector, ground],
            params(&[
                ("current_transfer_ratio", ctr),
                ("forward_voltage", 1.2),
                ("rs", 10.0),
                ("led_leakage", 1e-9),
                ("vce_sat", 0.2),
                ("leakage", 1e-9),
            ]),
        )
        .expect("U1");
    builder.build().expect("circuit")
}

#[test]
fn optocoupler_switches_with_led_current() {
    let on = solve(&optocoupler_circuit(5.0, 1.0));
    let off = solve(&optocoupler_circuit(0.0, 1.0));
    let on_collector = on.solution.voltage(NodeId::new(4)).expect("on");
    let off_collector = off.solution.voltage(NodeId::new(4)).expect("off");
    assert!(on_collector < 2.5, "on={on_collector}");
    assert!(off_collector > 4.5, "off={off_collector}");
}

#[test]
fn higher_ctr_pulls_collector_lower() {
    let low = solve(&optocoupler_circuit(5.0, 1.0));
    let high = solve(&optocoupler_circuit(5.0, 2.0));
    let low_collector = low.solution.voltage(NodeId::new(4)).expect("low");
    let high_collector = high.solution.voltage(NodeId::new(4)).expect("high");
    assert!(
        high_collector < low_collector,
        "low={low_collector} high={high_collector}"
    );
}

fn sensor_circuit(kind: DeviceKind, values: &[(&str, f64)]) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device("U1", kind, vec![supply, ground, output], params(values))
        .expect("U1");
    builder.build().expect("circuit")
}

fn sensor_output(kind: DeviceKind, values: &[(&str, f64)]) -> f64 {
    let result = solve(&sensor_circuit(kind, values));
    result.solution.voltage(NodeId::new(2)).expect("output")
}

#[test]
fn ntc_and_ptc_follow_temperature() {
    let ntc_cold = sensor_output(DeviceKind::Ntc, &[("temperature", 25.0)]);
    let ntc_hot = sensor_output(DeviceKind::Ntc, &[("temperature", 100.0)]);
    let ptc_cold = sensor_output(DeviceKind::Ptc, &[("temperature", 25.0)]);
    let ptc_hot = sensor_output(DeviceKind::Ptc, &[("temperature", 100.0)]);
    assert!((ntc_cold - 2.5).abs() < 0.1, "ntc_cold={ntc_cold}");
    assert!(ntc_hot < ntc_cold, "ntc_hot={ntc_hot}");
    assert!(ptc_hot > ptc_cold, "ptc_hot={ptc_hot}");
}

#[test]
fn rtd_matches_pt100_divider() {
    let zero = sensor_output(DeviceKind::Rtd, &[("temperature", 0.0)]);
    let hundred = sensor_output(DeviceKind::Rtd, &[("temperature", 100.0)]);
    assert!((zero - 100.0 / 1100.0 * 5.0).abs() < 0.01, "zero={zero}");
    assert!(
        (hundred - 138.5 / 1138.5 * 5.0).abs() < 0.02,
        "hundred={hundred}"
    );
}

#[test]
fn thermocouple_and_semiconductor_outputs_are_linear() {
    let thermocouple = sensor_output(
        DeviceKind::Thermocouple,
        &[("temperature", 100.0), ("cold_junction", 25.0)],
    );
    let semiconductor = sensor_output(DeviceKind::TemperatureSensor, &[("temperature", 25.0)]);
    assert!(
        (thermocouple - 41e-6 * 75.0).abs() < 1e-4,
        "tc={thermocouple}"
    );
    assert!(
        (semiconductor - 0.25).abs() < 0.01,
        "sensor={semiconductor}"
    );
}

#[test]
fn thermal_switch_switches_at_threshold() {
    let hot = sensor_output(DeviceKind::ThermalSwitch, &[("temperature", 80.0)]);
    let cold = sensor_output(DeviceKind::ThermalSwitch, &[("temperature", 60.0)]);
    assert!((hot - 5.0).abs() < 0.01, "hot={hot}");
    assert!(cold.abs() < 0.01, "cold={cold}");
}

#[test]
fn photoresistor_and_photodiode_respond_to_light() {
    let dim = sensor_output(DeviceKind::Photoresistor, &[("illuminance", 10.0)]);
    let bright = sensor_output(DeviceKind::Photoresistor, &[("illuminance", 100.0)]);
    let diode = sensor_output(DeviceKind::PhotodiodeSensor, &[("irradiance", 1.0)]);
    assert!((dim - 2.5).abs() < 0.1, "dim={dim}");
    assert!(bright < dim, "bright={bright}");
    assert!((diode - 0.5).abs() < 0.01, "diode={diode}");
}

#[test]
fn linear_light_sensor_scales_with_illuminance() {
    let first = sensor_output(
        DeviceKind::LinearLightSensor,
        &[("sensitivity", 0.001), ("illuminance", 1000.0)],
    );
    let second = sensor_output(
        DeviceKind::LinearLightSensor,
        &[("sensitivity", 0.001), ("illuminance", 2000.0)],
    );
    assert!((first - 1.0).abs() < 0.01, "first={first}");
    assert!((second - 2.0).abs() < 0.01, "second={second}");
}

#[test]
fn hall_and_reed_switches_respond_to_field() {
    let hall_off = sensor_output(DeviceKind::HallSwitch, &[("magnetic_field", 0.0)]);
    let hall_on = sensor_output(DeviceKind::HallSwitch, &[("magnetic_field", 0.01)]);
    let reed_on = sensor_output(DeviceKind::ReedSwitchSensor, &[("magnetic_field", 0.01)]);
    assert!(hall_off.abs() < 0.01, "hall_off={hall_off}");
    assert!((hall_on - 5.0).abs() < 0.01, "hall_on={hall_on}");
    assert!((reed_on - 5.0).abs() < 0.01, "reed_on={reed_on}");
}

#[test]
fn linear_magnetic_sensors_scale_with_field() {
    let hall = sensor_output(DeviceKind::LinearHallSensor, &[("magnetic_field", 0.2)]);
    let fluxgate = sensor_output(DeviceKind::FluxgateSensor, &[("magnetic_field", 0.001)]);
    let bridge = sensor_output(
        DeviceKind::MagnetoresistiveSensor,
        &[("magnetic_field", 0.05)],
    );
    assert!((hall - 3.5).abs() < 0.01, "hall={hall}");
    assert!((fluxgate - 1.0).abs() < 0.01, "fluxgate={fluxgate}");
    assert!((bridge - 3.0).abs() < 0.01, "bridge={bridge}");
}

#[test]
fn motion_sensors_scale_with_environment_inputs() {
    let accel = sensor_output(DeviceKind::Accelerometer, &[("acceleration", 9.80665)]);
    let gyro = sensor_output(DeviceKind::Gyroscope, &[("angular_rate", 100.0)]);
    let imu = sensor_output(DeviceKind::Imu, &[("acceleration", 9.80665)]);
    let tilt = sensor_output(DeviceKind::TiltSensor, &[("angle", 20.0)]);
    let vibration = sensor_output(DeviceKind::VibrationSensor, &[("vibration", 9.80665)]);
    assert!((accel - 3.0).abs() < 0.01, "accel={accel}");
    assert!((gyro - 3.5).abs() < 0.01, "gyro={gyro}");
    assert!((imu - 3.0).abs() < 0.01, "imu={imu}");
    assert!((tilt - 3.0).abs() < 0.01, "tilt={tilt}");
    assert!((vibration - 3.0).abs() < 0.01, "vibration={vibration}");
}

#[test]
fn strain_gauge_and_load_cell_use_excitation() {
    let strain = sensor_output(DeviceKind::StrainGaugeSensor, &[("strain", 0.001)]);
    let load = sensor_output(DeviceKind::LoadCell, &[("load", 50.0)]);
    assert!((strain - 0.002625).abs() < 1e-6, "strain={strain}");
    assert!((load - 0.005).abs() < 1e-6, "load={load}");
}

#[test]
fn pressure_and_force_sensors_scale_linearly() {
    let pressure = sensor_output(DeviceKind::PressureSensor, &[("pressure", 100000.0)]);
    let force = sensor_output(DeviceKind::ForceSensor, &[("force", 100.0)]);
    assert!((pressure - 1.5).abs() < 0.01, "pressure={pressure}");
    assert!((force - 1.0).abs() < 0.01, "force={force}");
}

#[test]
fn humidity_gas_ph_and_electrochemical_sensors_respond() {
    let humidity = sensor_output(DeviceKind::HumiditySensor, &[("relative_humidity", 50.0)]);
    let gas = sensor_output(DeviceKind::GasSensor, &[("gas_concentration", 1000.0)]);
    let ph = sensor_output(DeviceKind::PhSensor, &[("ph", 4.0)]);
    let chemical = sensor_output(
        DeviceKind::ElectrochemicalSensor,
        &[("concentration", 100.0)],
    );
    assert!((humidity - 2.3).abs() < 0.01, "humidity={humidity}");
    assert!((gas - 2.3684).abs() < 0.01, "gas={gas}");
    assert!((ph - 2.67748).abs() < 0.01, "ph={ph}");
    assert!((chemical - 1.0).abs() < 0.01, "chemical={chemical}");
}

#[test]
fn current_and_voltage_sensors_respond() {
    let shunt = sensor_output(DeviceKind::CurrentSenseResistor, &[("current", 1.0)]);
    let hall = sensor_output(DeviceKind::HallCurrentSensor, &[("current", 10.0)]);
    let divider = sensor_output(DeviceKind::VoltageDividerSensor, &[("voltage", 10.0)]);
    assert!((shunt - 2.0).abs() < 0.01, "shunt={shunt}");
    assert!((hall - 3.5).abs() < 0.01, "hall={hall}");
    assert!((divider - 2.0).abs() < 0.01, "divider={divider}");
}

#[test]
fn proximity_sensors_respond_to_distance_and_capacitance() {
    let inductive = sensor_output(DeviceKind::InductiveProximitySensor, &[("distance", 1.0)]);
    let touch = sensor_output(DeviceKind::CapacitiveTouchSensor, &[("capacitance", 20.0)]);
    let ultrasonic = sensor_output(DeviceKind::UltrasonicDistanceSensor, &[("distance", 2.0)]);
    assert!((inductive - 5.0).abs() < 0.01, "inductive={inductive}");
    assert!((touch - 5.0).abs() < 0.01, "touch={touch}");
    assert!((ultrasonic - 2.5).abs() < 0.01, "ultrasonic={ultrasonic}");
}

fn series_load_circuit(kind: DeviceKind, values: &[(&str, f64)], series: f64) -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, output],
            params(&[("resistance", series)]),
        )
        .expect("R1");
    builder
        .device("A1", kind, vec![output, ground], params(values))
        .expect("A1");
    builder.build().expect("circuit")
}

fn load_voltage(kind: DeviceKind, values: &[(&str, f64)], series: f64) -> f64 {
    let result = solve(&series_load_circuit(kind, values, series));
    result.solution.voltage(NodeId::new(2)).expect("output")
}

#[test]
fn led_actuator_forward_voltage_is_led_like() {
    let voltage = load_voltage(DeviceKind::LedActuator, &[("is", 1e-15)], 220.0);
    assert!((1.5..3.0).contains(&voltage), "voltage={voltage}");
}

#[test]
fn motor_current_depends_on_back_emf() {
    let voltage = load_voltage(
        DeviceKind::DcMotor,
        &[("resistance", 10.0), ("speed", 100.0)],
        1.0,
    );
    assert!((voltage - 4.63636).abs() < 0.05, "voltage={voltage}");
}

#[test]
fn peltier_current_depends_on_temperature_difference() {
    let voltage = load_voltage(
        DeviceKind::Peltier,
        &[
            ("resistance", 2.0),
            ("delta_temperature", 20.0),
            ("seebeck", 0.05),
        ],
        0.5,
    );
    assert!((voltage - 4.2).abs() < 0.05, "voltage={voltage}");
}

#[test]
fn thermoelectric_generator_drives_a_load() {
    let mut builder = CircuitBuilder::new();
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    builder
        .device(
            "G1",
            DeviceKind::ThermoelectricGenerator,
            vec![output, ground],
            params(&[
                ("resistance", 5.0),
                ("seebeck", 0.05),
                ("delta_temperature", 40.0),
            ]),
        )
        .expect("G1");
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![output, ground],
            params(&[("resistance", 5.0)]),
        )
        .expect("R1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let voltage = result.solution.voltage(NodeId::new(1)).expect("output");
    assert!((voltage - 1.0).abs() < 0.01, "voltage={voltage}");
}

#[test]
fn display_module_supply_current_scales_with_brightness() {
    let full = display_voltage(1.0);
    let half = display_voltage(0.5);
    assert!((full - 4.80769).abs() < 0.01, "full={full}");
    assert!((half - 4.90196).abs() < 0.01, "half={half}");
}

fn display_voltage(brightness: f64) -> f64 {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, output],
            params(&[("resistance", 100.0)]),
        )
        .expect("R1");
    builder
        .device(
            "D1",
            DeviceKind::DisplayModule,
            vec![output, ground, ground, ground],
            params(&[("brightness", brightness)]),
        )
        .expect("D1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    result.solution.voltage(NodeId::new(2)).expect("output")
}

#[test]
fn rf_amplifier_applies_voltage_gain() {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("in").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", input, ground, 1.0);
    builder
        .device(
            "A1",
            DeviceKind::RfAmplifier,
            vec![input, output, ground],
            params(&[("gain", 10.0)]),
        )
        .expect("A1");
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![output, ground],
            params(&[("resistance", 1000.0)]),
        )
        .expect("R1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let voltage = result.solution.voltage(NodeId::new(2)).expect("output");
    assert!((voltage - 10.0).abs() < 0.01, "voltage={voltage}");
}

#[test]
fn dcdc_converter_regulates_output_and_draws_input_current() {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vin").expect("node");
    let input = builder.node("in").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 12.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, input],
            params(&[("resistance", 1.0)]),
        )
        .expect("R1");
    builder
        .device(
            "U1",
            DeviceKind::DcDcConverter,
            vec![input, ground, output, ground],
            params(&[("output_voltage", 5.0), ("efficiency", 0.85)]),
        )
        .expect("U1");
    builder
        .device(
            "R2",
            DeviceKind::Resistor,
            vec![output, ground],
            params(&[("resistance", 10.0)]),
        )
        .expect("R2");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let input_voltage = result.solution.voltage(input).expect("input");
    let output_voltage = result.solution.voltage(output).expect("output");
    assert!((output_voltage - 5.0).abs() < 0.01, "out={output_voltage}");
    assert!((input_voltage - 11.7549).abs() < 0.02, "in={input_voltage}");
}

#[test]
fn charge_pump_regulates_output_and_draws_input_current() {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vin").expect("node");
    let input = builder.node("in").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, input],
            params(&[("resistance", 1.0)]),
        )
        .expect("R1");
    builder
        .device(
            "U1",
            DeviceKind::ChargePump,
            vec![input, output, ground],
            params(&[("output_voltage", 5.0), ("efficiency", 0.8)]),
        )
        .expect("U1");
    builder
        .device(
            "R2",
            DeviceKind::Resistor,
            vec![output, ground],
            params(&[("resistance", 10.0)]),
        )
        .expect("R2");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let input_voltage = result.solution.voltage(input).expect("input");
    let output_voltage = result.solution.voltage(output).expect("output");
    assert!((output_voltage - 5.0).abs() < 0.01, "out={output_voltage}");
    assert!((input_voltage - 4.375).abs() < 0.02, "in={input_voltage}");
}

#[test]
fn voltage_reference_outputs_threshold_and_draws_quiescent_current() {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vin").expect("node");
    let reference = builder.node("ref").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, reference],
            params(&[("resistance", 1000.0)]),
        )
        .expect("R1");
    builder
        .device(
            "U1",
            DeviceKind::VoltageReference,
            vec![reference, ground, output],
            params(&[("threshold", 2.5), ("quiescent_current", 1e-3)]),
        )
        .expect("U1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let reference_voltage = result.solution.voltage(reference).expect("ref");
    let output_voltage = result.solution.voltage(output).expect("out");
    assert!(
        (reference_voltage - 4.0).abs() < 0.01,
        "ref={reference_voltage}"
    );
    assert!((output_voltage - 2.5).abs() < 0.01, "out={output_voltage}");
}

#[test]
fn current_reference_drives_a_load() {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vin").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", supply, ground, 5.0);
    builder
        .device(
            "U1",
            DeviceKind::CurrentReference,
            vec![supply, ground, output],
            params(&[("output_current", 1e-3)]),
        )
        .expect("U1");
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![output, ground],
            params(&[("resistance", 1000.0)]),
        )
        .expect("R1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    let voltage = result.solution.voltage(output).expect("out");
    assert!((voltage - 1.0).abs() < 0.01, "out={voltage}");
}

#[test]
fn supervisor_polarity_follows_supply() {
    let high = supervisor_output(5.0, 0.0);
    let low = supervisor_output(2.0, 0.0);
    let reset_high = supervisor_output(5.0, 1.0);
    let reset_low = supervisor_output(2.0, 1.0);
    assert!((high - 5.0).abs() < 0.01, "high={high}");
    assert!(low.abs() < 0.01, "low={low}");
    assert!(reset_high.abs() < 0.01, "reset_high={reset_high}");
    assert!((reset_low - 2.0).abs() < 0.01, "reset_low={reset_low}");
}

fn supervisor_output(supply: f64, active_low: f64) -> f64 {
    let mut builder = CircuitBuilder::new();
    let input = builder.node("vin").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    add_input(&mut builder, "V1", input, ground, supply);
    builder
        .device(
            "U1",
            DeviceKind::Supervisor,
            vec![input, ground, output],
            params(&[("threshold", 2.5), ("active_low", active_low)]),
        )
        .expect("U1");
    let circuit = builder.build().expect("circuit");
    let result = solve(&circuit);
    result.solution.voltage(output).expect("out")
}
