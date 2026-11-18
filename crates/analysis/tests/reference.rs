//! Analytic reference circuits for release verification.

use analysis::{DcConfig, TransientConfig, dc_operating_point, transient};
use kernel::{Circuit, CircuitBuilder, DeviceKind, NodeId};
use std::collections::BTreeMap;

fn params(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_string(), *value))
        .collect()
}

fn divider() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    builder
        .device(
            "V1",
            DeviceKind::VoltageSource,
            vec![supply, ground],
            params(&[("voltage", 5.0)]),
        )
        .expect("V1");
    for (name, top, bottom) in [("R1", supply, output), ("R2", output, ground)] {
        builder
            .device(
                name,
                DeviceKind::Resistor,
                vec![top, bottom],
                params(&[("resistance", 1000.0)]),
            )
            .expect("resistor");
    }
    builder.build().expect("circuit")
}

fn rc() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    builder
        .device(
            "V1",
            DeviceKind::VoltageSource,
            vec![supply, ground],
            params(&[("voltage", 5.0)]),
        )
        .expect("V1");
    builder
        .device(
            "R1",
            DeviceKind::Resistor,
            vec![supply, output],
            params(&[("resistance", 1000.0)]),
        )
        .expect("R1");
    builder
        .device(
            "C1",
            DeviceKind::Capacitor,
            vec![output, ground],
            params(&[("capacitance", 1e-6)]),
        )
        .expect("C1");
    builder.build().expect("circuit")
}

fn rl() -> Circuit {
    let mut builder = CircuitBuilder::new();
    let supply = builder.node("vcc").expect("node");
    let output = builder.node("out").expect("node");
    let ground = builder.ground();
    builder
        .device(
            "V1",
            DeviceKind::VoltageSource,
            vec![supply, ground],
            params(&[("voltage", 5.0)]),
        )
        .expect("V1");
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
            "L1",
            DeviceKind::Inductor,
            vec![output, ground],
            params(&[("inductance", 1e-3)]),
        )
        .expect("L1");
    builder.build().expect("circuit")
}

#[test]
fn divider_matches_analytic_solution() {
    let result = dc_operating_point(&divider(), &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(NodeId::new(2)).expect("out");
    assert!((voltage - 2.5).abs() < 1e-6, "voltage={voltage}");
}

#[test]
fn rc_step_matches_analytic_solution_at_one_time_constant() {
    let config = TransientConfig {
        dt: 1e-6,
        steps: 1000,
        use_dc_op: false,
        ..TransientConfig::default()
    };
    let samples = transient(&rc(), &config).expect("transient");
    let voltage = samples[999].solution.voltage(NodeId::new(2)).expect("out");
    let expected = 5.0 * (1.0 - (-1.0_f64).exp());
    assert!(
        (voltage - expected).abs() < 0.02,
        "voltage={voltage} expected={expected}"
    );
}

#[test]
fn rl_current_matches_analytic_solution_at_one_time_constant() {
    let config = TransientConfig {
        dt: 1e-7,
        steps: 100,
        use_dc_op: false,
        ..TransientConfig::default()
    };
    let samples = transient(&rl(), &config).expect("transient");
    let current = samples[99].device_currents[2].expect("inductor current");
    let expected = 5.0 / 100.0 * (1.0 - (-1.0_f64).exp());
    assert!(
        (current - expected).abs() < 1e-3,
        "current={current} expected={expected}"
    );
}
