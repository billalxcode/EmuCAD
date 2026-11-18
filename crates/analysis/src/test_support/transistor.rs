use super::*;

pub(crate) fn common_emitter() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let base_drive = circuit.node("base_drive");
    let base = circuit.node("base");
    let collector = circuit.node("collector");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vb", base_drive, ground, 2.0);
    circuit.resistor("rb", base_drive, base, 100_000.0);
    circuit.resistor("rc", vcc, collector, 1000.0);
    circuit.bjt("q1", DeviceKind::Npn, collector, base, ground, 100.0);
    (circuit.build(), collector)
}

pub(crate) fn common_emitter_pnp() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let base = circuit.node("base");
    let collector = circuit.node("collector");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vb", base, ground, 4.3);
    circuit.resistor("rc", collector, ground, 1000.0);
    circuit.bjt("q1", DeviceKind::Pnp, collector, base, vcc, 100.0);
    (circuit.build(), collector)
}

pub(crate) fn common_source_n() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.resistor("rd", vcc, drain, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetN, drain, ground, ground, (-2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn common_source_n_biased() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let gate = circuit.node("gate");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.voltage("vg", gate, ground, -1.0);
    circuit.resistor("rd", vcc, drain, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetN, drain, gate, ground, (-2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn common_source_p() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vcc = circuit.node("vcc");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vcc_src", vcc, ground, 5.0);
    circuit.resistor("rd", drain, ground, 1000.0);
    circuit.jfet("j1", DeviceKind::JfetP, drain, vcc, vcc, (2.0, 1e-3));
    (circuit.build(), drain)
}

pub(crate) fn matched_line() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let source = circuit.node("source");
    let input = circuit.node("input");
    let output = circuit.node("output");
    let ground = circuit.ground();
    circuit.voltage("vs", source, ground, 5.0);
    circuit.resistor("rs", source, input, 50.0);
    circuit.resistor("rl", output, ground, 50.0);
    circuit.transmission_line("t1", [input, ground, output, ground], (50.0, 1e-6));
    (circuit.build(), output)
}

pub(crate) fn common_source_body() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vdd = circuit.node("vdd");
    let gate = circuit.node("gate");
    let body = circuit.node("body");
    let drain = circuit.node("drain");
    let ground = circuit.ground();
    circuit.voltage("vdd_src", vdd, ground, 5.0);
    circuit.voltage("vg", gate, ground, 1.5);
    circuit.voltage("vb", body, ground, 0.0);
    circuit.resistor("rd", vdd, drain, 1000.0);
    circuit.mosfet_body("m1", DeviceKind::Nmos, [gate, drain, ground, body], 10e-6);
    (circuit.build(), drain)
}

pub(crate) fn mos_capacitor() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let gate = circuit.node("gate");
    let ground = circuit.ground();
    circuit.pulse(
        "v1",
        input,
        ground,
        (0.0, 5.0),
        (1e-6, 0.0, 0.0, 10e-6, 20e-6),
    );
    circuit.resistor("rg", input, gate, 1e6);
    add_mos_capacitor(&mut circuit, gate, ground);
    (circuit.build(), gate)
}

fn add_mos_capacitor(circuit: &mut TestCircuit, gate: NodeId, ground: NodeId) {
    let mut parameters = mos_params(10e-6);
    parameters.insert("cox".to_string(), 1e-12);
    parameters.insert("gamma".to_string(), 0.0);
    circuit
        .builder
        .device(
            "m1",
            DeviceKind::Nmos,
            vec![gate, ground, ground, ground],
            parameters,
        )
        .expect("m1");
}
