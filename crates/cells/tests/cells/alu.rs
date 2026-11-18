use super::*;

fn is_high(voltage: f64) -> bool {
    voltage > 2.5
}

fn alu_outputs(first: f64, second: f64, carry_in: f64, op_low: f64, op_high: f64) -> (f64, f64) {
    let cell = alu_slice().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    add_source(&mut circuit, "va", cell.port("a").expect("a"), first).expect("va");
    add_source(&mut circuit, "vb", cell.port("b").expect("b"), second).expect("vb");
    add_source(&mut circuit, "vc", cell.port("cin").expect("cin"), carry_in).expect("vc");
    add_source(&mut circuit, "vo0", cell.port("op0").expect("op0"), op_low).expect("vo0");
    add_source(&mut circuit, "vo1", cell.port("op1").expect("op1"), op_high).expect("vo1");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let output = result
        .solution
        .voltage(cell.port("y").expect("y"))
        .unwrap_or(0.0);
    let carry = result
        .solution
        .voltage(cell.port("carry").expect("carry"))
        .unwrap_or(0.0);
    (output, carry)
}

#[test]
fn alu_slice_logic_truth_table() {
    let cases = [
        (0.0, 0.0, false, false, false),
        (0.0, 5.0, false, true, true),
        (5.0, 0.0, false, true, true),
        (5.0, 5.0, true, true, false),
    ];
    for (a, b, expected_and, expected_or, expected_xor) in cases {
        assert_eq!(is_high(alu_outputs(a, b, 0.0, 0.0, 0.0).0), expected_and);
        assert_eq!(is_high(alu_outputs(a, b, 0.0, 5.0, 0.0).0), expected_or);
        assert_eq!(is_high(alu_outputs(a, b, 0.0, 0.0, 5.0).0), expected_xor);
    }
}

#[test]
fn alu_slice_addition_truth_table() {
    let cases = [
        (0.0, 0.0, 0.0, false, false),
        (0.0, 5.0, 0.0, true, false),
        (5.0, 0.0, 0.0, true, false),
        (5.0, 5.0, 0.0, false, true),
        (0.0, 0.0, 5.0, true, false),
        (5.0, 5.0, 5.0, true, true),
    ];
    for (a, b, carry_in, expected_sum, expected_carry) in cases {
        let (sum, carry) = alu_outputs(a, b, carry_in, 5.0, 5.0);
        assert_eq!(is_high(sum), expected_sum);
        assert_eq!(is_high(carry), expected_carry);
    }
}
