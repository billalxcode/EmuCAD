use super::{DcConfig, dc_operating_point, dc_sweep};
use crate::test_support::{
    cccs_circuit, ccvs_circuit, common_emitter, common_emitter_pnp, common_source_body,
    common_source_n, common_source_n_biased, common_source_p, diode_clamp, floating_node, inverter,
    rc_circuit, rl_circuit, switch_circuit, vccs_circuit, vcvs_circuit, voltage_divider,
};

#[test]
fn voltage_divider_is_half_supply() {
    let (circuit, output) = voltage_divider();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 5.0).abs() < 1e-6);
    assert!(result.residual.is_finite() && result.residual < 1e-3);
}

#[test]
fn dc_sweep_visits_expected_points() {
    let (circuit, output) = voltage_divider();
    let points = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        0.0,
        10.0,
        5.0,
        &DcConfig::default(),
    )
    .expect("sweep");
    assert_eq!(points.len(), 3);
    let middle = points[1].solution.voltage(output).expect("out");
    assert!((middle - 2.5).abs() < 1e-6);
}

#[test]
fn dc_sweep_rejects_zero_step() {
    let (circuit, _) = voltage_divider();
    let result = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        0.0,
        1.0,
        0.0,
        &DcConfig::default(),
    );
    assert!(result.is_err());
}

#[test]
fn dc_sweep_rejects_reverse_range() {
    let (circuit, _) = voltage_divider();
    let result = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        5.0,
        1.0,
        1.0,
        &DcConfig::default(),
    );
    assert!(result.is_err());
}

#[test]
fn dc_sweep_rejects_non_finite_step() {
    let (circuit, _) = voltage_divider();
    let result = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        0.0,
        1.0,
        f64::NAN,
        &DcConfig::default(),
    );
    assert!(result.is_err());
}

#[test]
fn dc_config_default_is_reasonable() {
    let config = DcConfig::default();
    assert!(config.max_iterations > 0);
    assert!(config.rel_tol > 0.0);
    assert!(config.abs_tol > 0.0);
    assert!(config.gmin > 0.0);
}

#[test]
fn inverter_vtc_switches() {
    let (circuit, _, output) = inverter();
    let points = dc_sweep(
        &circuit,
        "vin",
        "voltage",
        0.0,
        5.0,
        0.5,
        &DcConfig::default(),
    )
    .expect("sweep");
    let first = points.first().expect("first");
    let last = points.last().expect("last");
    assert!(first.solution.voltage(output).expect("high") > 4.0);
    assert!(last.solution.voltage(output).expect("low") < 1.0);
}

#[test]
fn inverter_operating_point_converges() {
    let (circuit, _, _) = inverter();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    assert!(result.iterations > 0);
}

#[test]
fn capacitor_is_open_at_dc() {
    let (circuit, output) = rc_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 5.0).abs() < 1e-6);
}

#[test]
fn inductor_is_short_at_dc() {
    let (circuit, output) = rl_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage.abs() < 1e-6);
}

#[test]
fn dc_sweep_unknown_device_is_rejected() {
    let (circuit, _) = voltage_divider();
    let result = dc_sweep(
        &circuit,
        "missing",
        "voltage",
        0.0,
        1.0,
        0.5,
        &DcConfig::default(),
    );
    assert!(result.is_err());
}

#[test]
fn dc_sweep_single_point_has_solution() {
    let (circuit, _) = voltage_divider();
    let points = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        3.0,
        3.0,
        1.0,
        &DcConfig::default(),
    )
    .expect("sweep");
    assert_eq!(points.len(), 1);
    assert!((points[0].sweep - 3.0).abs() < 1e-12);
}

#[test]
fn dc_sweep_tracks_previous_solution() {
    let (circuit, output) = diode_clamp();
    let points = dc_sweep(
        &circuit,
        "v1",
        "voltage",
        0.1,
        2.0,
        0.1,
        &DcConfig::default(),
    )
    .expect("sweep");
    assert_eq!(points.len(), 20);
    let first = points.first().expect("first");
    let last = points.last().expect("last");
    let first = first.solution.voltage(output).expect("first v");
    let last = last.solution.voltage(output).expect("last v");
    assert!(last > first + 0.1, "first={first} last={last}");
}

#[test]
fn diode_clamp_limits_voltage() {
    let (circuit, output) = diode_clamp();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 0.4 && voltage < 0.9, "voltage={voltage}");
}

#[test]
fn diode_clamp_converges_from_large_supply() {
    let (mut circuit, output) = diode_clamp();
    circuit
        .set_device_parameter("v1", "voltage", 20.0)
        .expect("v1");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 0.4 && voltage < 0.95, "voltage={voltage}");
}

#[test]
fn diode_series_resistance_raises_clamp_voltage() {
    let (mut circuit, output) = diode_clamp();
    circuit.set_device_parameter("d1", "rs", 100.0).expect("rs");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 0.9 && voltage < 1.3, "voltage={voltage}");
}

#[test]
fn source_stepping_recovers_large_diode_supply() {
    let (mut circuit, output) = diode_clamp();
    circuit
        .set_device_parameter("v1", "voltage", 100.0)
        .expect("v1");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 0.5 && voltage < 0.9, "voltage={voltage}");
}

#[test]
fn npn_common_emitter_conducts() {
    let (circuit, output) = common_emitter();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
}

#[test]
fn higher_beta_lowers_collector_voltage() {
    let (mut circuit, output) = common_emitter();
    circuit.set_device_parameter("q1", "bf", 200.0).expect("bf");
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage < 3.0, "voltage={voltage}");
}

#[test]
fn pnp_common_emitter_conducts() {
    let (circuit, output) = common_emitter_pnp();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 0.2 && voltage < 1.5, "voltage={voltage}");
}

#[test]
fn n_jfet_common_source_conducts() {
    let (circuit, output) = common_source_n();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 1.0 && voltage < 1.8, "voltage={voltage}");
}

#[test]
fn negative_gate_bias_raises_jfet_drain_voltage() {
    let (base, base_output) = common_source_n();
    let (biased, biased_output) = common_source_n_biased();
    let base = dc_operating_point(&base, &DcConfig::default()).expect("dc");
    let biased = dc_operating_point(&biased, &DcConfig::default()).expect("dc");
    let base = base.solution.voltage(base_output).expect("base");
    let biased = biased.solution.voltage(biased_output).expect("biased");
    assert!(biased > base + 0.2, "base={base} biased={biased}");
}

#[test]
fn p_jfet_common_source_conducts() {
    let (circuit, output) = common_source_p();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
}

#[test]
fn body_bias_raises_drain_voltage() {
    let (mut circuit, output) = common_source_body();
    let base = dc_operating_point(&circuit, &DcConfig::default()).expect("base");
    let base = base.solution.voltage(output).expect("base v");
    circuit
        .set_device_parameter("vb", "voltage", -2.0)
        .expect("vb");
    let biased = dc_operating_point(&circuit, &DcConfig::default()).expect("biased");
    let biased = biased.solution.voltage(output).expect("biased v");
    assert!(biased > base + 0.1, "base={base} biased={biased}");
}

#[test]
fn vcvs_output_is_gain_times_control() {
    let (circuit, output) = vcvs_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 2.0).abs() < 1e-9, "voltage={voltage}");
}

#[test]
fn vccs_output_is_gm_times_control_times_load() {
    let (circuit, output) = vccs_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 1.0).abs() < 1e-9, "voltage={voltage}");
}

#[test]
fn cccs_output_is_gain_times_control_current() {
    let (circuit, output) = cccs_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 2.0).abs() < 1e-6, "voltage={voltage}");
}

#[test]
fn ccvs_output_is_rm_times_control_current() {
    let (circuit, output) = ccvs_circuit();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!((voltage - 1.0).abs() < 1e-6, "voltage={voltage}");
}

#[test]
fn switch_pulls_output_low_when_on() {
    let (on, on_output) = switch_circuit(true);
    let (off, off_output) = switch_circuit(false);
    let on = dc_operating_point(&on, &DcConfig::default()).expect("on");
    let off = dc_operating_point(&off, &DcConfig::default()).expect("off");
    let on = on.solution.voltage(on_output).expect("on v");
    let off = off.solution.voltage(off_output).expect("off v");
    assert!(on < 0.1, "on={on}");
    assert!(off > 4.9, "off={off}");
}

#[test]
fn floating_node_is_pinned_by_gmin() {
    let (circuit, output) = floating_node();
    let result = dc_operating_point(&circuit, &DcConfig::default()).expect("dc");
    let voltage = result.solution.voltage(output).expect("output");
    assert!(voltage.abs() < 1e-9, "voltage={voltage}");
}
