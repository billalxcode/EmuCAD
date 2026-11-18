use super::*;

fn half_adder_outputs(first: f64, second: f64) -> (f64, f64) {
    let cell = half_adder().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    let sum = output(&circuit, cell.port("sum").expect("sum"));
    let carry = output(&circuit, cell.port("carry").expect("carry"));
    (sum, carry)
}

#[test]
fn half_adder_truth_table() {
    let (sum, carry) = half_adder_outputs(0.0, 0.0);
    assert!(sum < 1.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(0.0, 5.0);
    assert!(sum > 4.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(5.0, 0.0);
    assert!(sum > 4.0 && carry < 1.0);
    let (sum, carry) = half_adder_outputs(5.0, 5.0);
    assert!(sum < 1.0 && carry > 4.0);
}
fn full_adder_outputs(first: f64, second: f64, carry_in: f64) -> (f64, f64) {
    let cell = full_adder().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let a = cell.port("a").expect("a");
    let b = cell.port("b").expect("b");
    let c = cell.port("cin").expect("cin");
    add_source(&mut circuit, "va", a, first).expect("va");
    add_source(&mut circuit, "vb", b, second).expect("vb");
    add_source(&mut circuit, "vc", c, carry_in).expect("vc");
    let sum = output(&circuit, cell.port("sum").expect("sum"));
    let carry = output(&circuit, cell.port("carry").expect("carry"));
    (sum, carry)
}

#[test]
fn full_adder_truth_table() {
    let cases = [
        (0.0, 0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 5.0, 5.0, 0.0),
        (0.0, 5.0, 0.0, 5.0, 0.0),
        (0.0, 5.0, 5.0, 0.0, 5.0),
        (5.0, 0.0, 0.0, 5.0, 0.0),
        (5.0, 0.0, 5.0, 0.0, 5.0),
        (5.0, 5.0, 0.0, 0.0, 5.0),
        (5.0, 5.0, 5.0, 5.0, 5.0),
    ];
    for (a, b, c, expected_sum, expected_carry) in cases {
        let (sum, carry) = full_adder_outputs(a, b, c);
        assert!((sum - expected_sum).abs() < 1.0);
        assert!((carry - expected_carry).abs() < 1.0);
    }
}
fn build_adder(bits: usize) -> (cells::Cell, Circuit) {
    let cell = ripple_carry_adder(bits).expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    for bit in 0..bits {
        add_adder_input(&mut circuit, &cell, bit);
    }
    add_source(&mut circuit, "vcin", cell.port("cin").expect("cin"), 0.0).expect("vcin");
    (cell, circuit)
}

fn add_adder_input(circuit: &mut Circuit, cell: &cells::Cell, bit: usize) {
    add_source(
        circuit,
        &format!("va{bit}"),
        cell.port(&format!("a{bit}")).expect("a"),
        0.0,
    )
    .expect("va");
    add_source(
        circuit,
        &format!("vb{bit}"),
        cell.port(&format!("b{bit}")).expect("b"),
        0.0,
    )
    .expect("vb");
}

fn apply_adder_inputs(circuit: &mut Circuit, bits: usize, first: u8, second: u8, carry: bool) {
    for bit in 0..bits {
        let a = if (first >> bit) & 1 == 1 { 5.0 } else { 0.0 };
        let b = if (second >> bit) & 1 == 1 { 5.0 } else { 0.0 };
        circuit
            .set_device_parameter(&format!("va{bit}"), "voltage", a)
            .expect("va");
        circuit
            .set_device_parameter(&format!("vb{bit}"), "voltage", b)
            .expect("vb");
    }
    let cin = if carry { 5.0 } else { 0.0 };
    circuit
        .set_device_parameter("vcin", "voltage", cin)
        .expect("vcin");
}

fn read_adder(circuit: &Circuit, cell: &cells::Cell, bits: usize) -> (u8, bool) {
    let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
    let mut sum = 0;
    for bit in 0..bits {
        let node = cell.port(&format!("sum{bit}")).expect("sum");
        if result.solution.voltage(node).unwrap_or(0.0) > 2.5 {
            sum |= 1 << bit;
        }
    }
    let carry = result
        .solution
        .voltage(cell.port("cout").expect("cout"))
        .unwrap_or(0.0)
        > 2.5;
    (sum, carry)
}

#[test]
fn four_bit_adder_truth_table() {
    let (cell, mut circuit) = build_adder(4);
    let cases = [
        (0, 0, false, 0, false),
        (3, 4, false, 7, false),
        (9, 6, false, 15, false),
        (15, 1, false, 0, true),
    ];
    for (first, second, carry_in, expected_sum, expected_carry) in cases {
        apply_adder_inputs(&mut circuit, 4, first, second, carry_in);
        let (sum, carry) = read_adder(&circuit, &cell, 4);
        assert_eq!(sum, expected_sum);
        assert_eq!(carry, expected_carry);
    }
}

#[test]
fn eight_bit_adder_has_ports() {
    let cell = adder8().expect("cell");
    assert!(cell.port("a7").is_some());
    assert!(cell.port("b7").is_some());
    assert!(cell.port("sum7").is_some());
    assert!(cell.port("cout").is_some());
}
