use super::*;

pub(crate) fn diode_rectifier() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.pulse(
        "v1",
        input,
        ground,
        (0.0, 5.0),
        (1e-6, 0.0, 0.0, 5e-6, 10e-6),
    );
    circuit.resistor("r1", input, output, 1000.0);
    circuit.two_pin("d1", DeviceKind::Diode, output, ground, BTreeMap::new());
    (circuit.build(), output)
}

pub(crate) fn floating_node() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let node = circuit.node("floating");
    let ground = circuit.ground();
    circuit.capacitor("c1", node, ground, 1e-6);
    let built = circuit.builder.build_topology().expect("circuit");
    (built, node)
}

pub(crate) fn reverse_diode_rc(cj0: f64) -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.pulse(
        "v1",
        input,
        ground,
        (0.0, -2.0),
        (0.0, 0.0, 0.0, 10e-6, 20e-6),
    );
    circuit.resistor("r1", input, output, 1000.0);
    let mut parameters = BTreeMap::new();
    parameters.insert("cj0".to_string(), cj0);
    circuit.two_pin("d1", DeviceKind::Diode, output, ground, parameters);
    (circuit.build(), output)
}

pub(crate) fn diode_varactor() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let cathode = circuit.node("cathode");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 0.0);
    circuit.voltage("vc", cathode, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    let mut parameters = BTreeMap::new();
    parameters.insert("cj0".to_string(), 10e-12);
    circuit.two_pin("d1", DeviceKind::Diode, output, cathode, parameters);
    (circuit.build(), output)
}

pub(crate) fn diode_clamp() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.diode("d1", output, ground);
    (circuit.build(), output)
}
