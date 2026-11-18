use super::*;

#[test]
fn inverter_vtc_switches() {
    let (output_port, points) = inverter_sweep();
    let first = points.first().expect("first");
    let last = points.last().expect("last");
    assert!(first.solution.voltage(output_port).expect("high") > 4.0);
    assert!(last.solution.voltage(output_port).expect("low") < 1.0);
}

fn nand_output(first: f64, second: f64) -> f64 {
    let cell = nand2().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn nand_truth_table() {
    assert!(nand_output(0.0, 0.0) > 4.0);
    assert!(nand_output(0.0, 5.0) > 4.0);
    assert!(nand_output(5.0, 0.0) > 4.0);
    assert!(nand_output(5.0, 5.0) < 1.0);
}

fn nor_output(first: f64, second: f64) -> f64 {
    let cell = nor2().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn nor_truth_table() {
    assert!(nor_output(0.0, 0.0) > 4.0);
    assert!(nor_output(0.0, 5.0) < 1.0);
    assert!(nor_output(5.0, 0.0) < 1.0);
    assert!(nor_output(5.0, 5.0) < 1.0);
}

#[test]
fn buffer_follows_input() {
    let cell = buffer().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    let output_port = cell.port("y").expect("y");
    add_source(&mut circuit, "vin", input, 0.0).expect("source");
    assert!(output(&circuit, output_port) < 1.0);
    circuit
        .set_device_parameter("vin", "voltage", 5.0)
        .expect("set input");
    assert!(output(&circuit, output_port) > 4.0);
}

#[test]
fn powered_cell_has_supply() {
    let cell = inverter().expect("cell");
    let circuit = cell.powered(5.0).expect("powered");
    assert!(circuit.device_by_name("vdd_src").is_some());
    assert_eq!(cell.vdd(), cell.port("vdd"));
}
#[test]
fn inverter_truth_table() {
    assert!(inverter_output(0.0) > 4.0);
    assert!(inverter_output(5.0) < 1.0);
}

#[test]
fn inverter_threshold_is_near_half_supply() {
    let (output_port, points) = inverter_sweep();
    let crossing = points
        .iter()
        .find(|point| point.solution.voltage(output_port).expect("out") < 2.5)
        .expect("crossing");
    assert!(crossing.sweep > 2.0 && crossing.sweep < 3.0);
}
fn gate_output(cell: cells::Cell, inputs: &[(&str, f64)]) -> f64 {
    let mut circuit = cell.powered(5.0).expect("powered");
    for (index, (port, voltage)) in inputs.iter().enumerate() {
        let node = cell.port(port).expect("input port");
        add_source(&mut circuit, &format!("v{index}"), node, *voltage).expect("source");
    }
    output(&circuit, cell.port("y").expect("y"))
}

#[test]
fn and_gate_truth_table() {
    assert!(gate_output(and2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(and2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) > 4.0);
}

#[test]
fn or_gate_truth_table() {
    assert!(gate_output(or2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) > 4.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) > 4.0);
    assert!(gate_output(or2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) > 4.0);
}

#[test]
fn xor_gate_truth_table() {
    assert!(gate_output(xor2().expect("cell"), &[("a", 0.0), ("b", 0.0)]) < 1.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 0.0), ("b", 5.0)]) > 4.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 5.0), ("b", 0.0)]) > 4.0);
    assert!(gate_output(xor2().expect("cell"), &[("a", 5.0), ("b", 5.0)]) < 1.0);
}

#[test]
fn mux_selects_inputs() {
    let cases = [
        (0.0, 0.0, 0.0, 0.0),
        (5.0, 0.0, 0.0, 5.0),
        (0.0, 5.0, 0.0, 0.0),
        (0.0, 0.0, 5.0, 0.0),
        (5.0, 0.0, 5.0, 0.0),
        (0.0, 5.0, 5.0, 5.0),
    ];
    for (first, second, select, expected) in cases {
        let result = gate_output(
            mux2().expect("cell"),
            &[("a", first), ("b", second), ("sel", select)],
        );
        assert!((result - expected).abs() < 1.0);
    }
}
