use super::*;

pub(crate) fn vcvs_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("vin", input, ground, 1.0);
    circuit.vcvs("e1", [output, ground], [input, ground], 2.0);
    circuit.resistor("rl", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn vccs_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("vin", input, ground, 1.0);
    circuit.vccs("g1", [output, ground], [input, ground], 1e-3);
    circuit.resistor("rl", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn cccs_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let control = circuit.node("control");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("vin", input, ground, 1.0);
    circuit.resistor("rc", input, control, 1000.0);
    circuit.cccs("f1", [output, ground], [control, ground], 2.0);
    circuit.resistor("rl", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn ccvs_circuit() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let control = circuit.node("control");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("vin", input, ground, 1.0);
    circuit.resistor("rc", input, control, 1000.0);
    circuit.ccvs("h1", [output, ground], [control, ground], 1000.0);
    circuit.resistor("rl", output, ground, 1000.0);
    (circuit.build(), output)
}

pub(crate) fn switch_circuit(on: bool) -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let input = circuit.node("in");
    let control = circuit.node("control");
    let output = circuit.node("out");
    let ground = circuit.ground();
    circuit.voltage("vcc", input, ground, 5.0);
    circuit.voltage("vc", control, ground, if on { 5.0 } else { 0.0 });
    circuit.resistor("r1", input, output, 1000.0);
    circuit.switch(
        "s1",
        [output, ground],
        [control, ground],
        (1.0, 1e9, 2.5, 0.1),
    );
    (circuit.build(), output)
}

pub(crate) fn switching_inverter() -> (Circuit, NodeId) {
    let mut circuit = TestCircuit::new();
    let vdd = circuit.supply("vdd");
    let input = circuit.node("input");
    let output = circuit.node("output");
    let ground = circuit.ground();
    circuit.voltage("vdd_src", vdd, ground, 5.0);
    circuit.pulse(
        "vin",
        input,
        ground,
        (0.0, 5.0),
        (1e-6, 0.0, 0.0, 2e-6, 4e-6),
    );
    circuit.mosfet("mp", DeviceKind::Pmos, input, output, vdd, 10e-6);
    circuit.mosfet("mn", DeviceKind::Nmos, input, output, ground, 5e-6);
    circuit.capacitor("cl", output, ground, 10e-12);
    (circuit.build(), output)
}
