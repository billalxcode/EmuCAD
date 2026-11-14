use super::*;

pub(crate) fn voltage_divider() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 10.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.resistor("r2", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn inverter() -> (Circuit, NodeId, NodeId) {
    let mut circuit = TestCircuit::new();
    let vdd = circuit.supply("vdd");
    let input = circuit.node("input");
    let output = circuit.node("output");
    let ground = circuit.ground();
    circuit.voltage("vdd_src", vdd, ground, 5.0);
    circuit.voltage("vin", input, ground, 0.0);
    circuit.mosfet("mp", DeviceKind::Pmos, input, output, vdd, 10e-6);
    circuit.mosfet("mn", DeviceKind::Nmos, input, output, ground, 5e-6);
    (circuit.build(), input, output)
}

pub(crate) fn rc_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.capacitor("c1", output, ground, 1e-6);
    (circuit.build(), output)
}

pub(crate) fn rc_circuit_no_source() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.resistor("r1", output, ground, 1000.0);
    circuit.capacitor("c1", output, ground, 1e-6);
    (circuit.build(), output)
}

pub(crate) fn rl_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("v1", input, ground, 5.0);
    circuit.resistor("r1", input, output, 1000.0);
    circuit.inductor("l1", output, ground, 1e-3);
    (circuit.build(), output)
}

pub(crate) fn current_source_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.current("i1", ground, output, 0.001);
    circuit.resistor("r1", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn pulse_divider() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.pulse("v1", input, ground, (0.0, 5.0), (0.0, 0.0, 0.0, 1e-6, 2e-6));
    circuit.resistor("r1", input, output, 1000.0);
    circuit.resistor("r2", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn pulse_rc() -> (Circuit, NodeId, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.pulse(
        "v1",
        input,
        ground,
        (0.0, 5.0),
        (5e-6, 0.0, 0.0, 10e-6, 20e-6),
    );
    circuit.resistor("r1", input, output, 1000.0);
    circuit.capacitor("c1", output, ground, 1e-9);
    (circuit.build(), input, output)
}

pub(crate) fn resistor_to_ground() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.resistor("r1", output, ground, 1000.0);
    (circuit.build(), output)
}
