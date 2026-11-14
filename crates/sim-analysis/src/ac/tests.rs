use super::{ac_analysis, ac_sweep};
use crate::test_support::{
    cccs_circuit, ccvs_circuit, common_emitter, common_source_n_biased, current_source_circuit,
    diode_clamp, diode_varactor, inverter, matched_line, pulse_divider, rc_circuit, rl_circuit,
    switch_circuit, vccs_circuit, vcvs_circuit,
};

#[test]
fn rc_low_pass_at_cutoff_has_expected_response() {
    let (mut circuit, output) = rc_circuit();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
    let result = ac_analysis(&circuit, cutoff).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    let phase = result.phase_degrees(output).expect("phase");
    assert!(
        (magnitude - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.01,
        "magnitude={magnitude}"
    );
    assert!((phase + 45.0).abs() < 1.0, "phase={phase}");
}

#[test]
fn voltage_source_branch_current_is_observable() {
    let (mut circuit, _) = rc_circuit();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let current = result.branch_current(0).expect("current");
    let magnitude = current.magnitude();
    assert!(
        magnitude > 9e-4 && magnitude < 1.1e-3,
        "current={current:?}"
    );
}

#[test]
fn inductor_branch_current_is_observable() {
    let (mut circuit, _) = rl_circuit();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1.0).expect("ac");
    let current = result.branch_current(2).expect("current");
    assert!(
        (current.magnitude() - 1e-3).abs() < 1e-5,
        "current={current:?}"
    );
}

#[test]
fn ac_sweep_returns_requested_points() {
    let (circuit, _) = rc_circuit();
    let points = ac_sweep(&circuit, 10.0, 1000.0, 5).expect("sweep");
    assert_eq!(points.len(), 5);
    assert!((points[0].frequency() - 10.0).abs() < 1e-12);
    assert!((points[4].frequency() - 1000.0).abs() < 1e-6);
}

#[test]
fn switch_state_controls_ac_response() {
    let (mut on, on_output) = switch_circuit(true);
    let (mut off, off_output) = switch_circuit(false);
    set_source_ac(&mut on, "vcc", 1.0);
    set_source_ac(&mut off, "vcc", 1.0);
    set_source_ac(&mut on, "vc", 0.0);
    set_source_ac(&mut off, "vc", 0.0);
    let on = ac_analysis(&on, 1000.0).expect("on");
    let off = ac_analysis(&off, 1000.0).expect("off");
    let on = on.magnitude(on_output).expect("on output");
    let off = off.magnitude(off_output).expect("off output");
    assert!(on < 0.01, "on={on}");
    assert!(off > 0.9, "off={off}");
}

#[test]
fn vcvs_output_tracks_ac_gain() {
    let (mut circuit, output) = vcvs_circuit();
    set_source_ac(&mut circuit, "vin", 1.0);
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("output");
    assert!((magnitude - 2.0).abs() < 1e-9, "magnitude={magnitude}");
}

#[test]
fn vccs_output_tracks_ac_transconductance() {
    let (mut circuit, output) = vccs_circuit();
    set_source_ac(&mut circuit, "vin", 1.0);
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("output");
    assert!((magnitude - 1.0).abs() < 1e-9, "magnitude={magnitude}");
}

#[test]
fn cccs_output_tracks_ac_gain() {
    let (mut circuit, output) = cccs_circuit();
    set_source_ac(&mut circuit, "vin", 1.0);
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("output");
    assert!((magnitude - 2.0).abs() < 1e-6, "magnitude={magnitude}");
}

#[test]
fn ccvs_output_tracks_ac_transresistance() {
    let (mut circuit, output) = ccvs_circuit();
    set_source_ac(&mut circuit, "vin", 1.0);
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("output");
    assert!((magnitude - 1.0).abs() < 1e-6, "magnitude={magnitude}");
    let sense = result.branch_current(2).expect("sense current");
    assert!((sense.magnitude() - 1e-3).abs() < 1e-6, "sense={sense:?}");
}

fn set_source_ac(circuit: &mut sim_core::Circuit, name: &str, value: f64) {
    circuit
        .set_device_parameter(name, "ac", value)
        .expect("source ac");
}

#[test]
fn ac_rejects_transmission_line() {
    let (circuit, _) = matched_line();
    assert!(ac_analysis(&circuit, 1000.0).is_err());
}

#[test]
fn ac_rejects_pulse_source() {
    let (circuit, _) = pulse_divider();
    assert!(ac_analysis(&circuit, 1000.0).is_err());
}

#[test]
fn rl_low_pass_at_cutoff_has_expected_response() {
    let (mut circuit, output) = rl_circuit();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    let cutoff = 1000.0 / (2.0 * std::f64::consts::PI * 1e-3);
    let result = ac_analysis(&circuit, cutoff).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    let phase = result.phase_degrees(output).expect("phase");
    assert!(
        (magnitude - std::f64::consts::FRAC_1_SQRT_2).abs() < 0.01,
        "magnitude={magnitude}"
    );
    assert!((phase - 45.0).abs() < 1.0, "phase={phase}");
}

#[test]
fn source_phase_rotates_response() {
    let (mut circuit, output) = rc_circuit();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    circuit
        .set_device_parameter("v1", "ac_phase", 90.0)
        .expect("phase");
    let cutoff = 1.0 / (2.0 * std::f64::consts::PI * 1000.0 * 1e-6);
    let result = ac_analysis(&circuit, cutoff).expect("ac");
    let phase = result.phase_degrees(output).expect("phase");
    assert!((phase - 45.0).abs() < 1.0, "phase={phase}");
}

#[test]
fn current_source_uses_current_as_ac_magnitude() {
    let (circuit, output) = current_source_circuit();
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    assert!((magnitude - 1.0).abs() < 1e-9, "magnitude={magnitude}");
}

#[test]
fn diode_small_signal_uses_dc_bias() {
    let (mut circuit, output) = diode_clamp();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    assert!(magnitude > 1e-6 && magnitude < 0.1, "magnitude={magnitude}");
}

#[test]
fn mosfet_inverter_has_gain() {
    let (mut circuit, input, output) = inverter();
    circuit
        .set_device_parameter("vin", "voltage", 2.5)
        .expect("bias");
    circuit.set_device_parameter("vin", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    assert!(magnitude > 1.0, "magnitude={magnitude}");
    assert!(result.magnitude(input).is_some());
}

#[test]
fn diode_capacitance_reduces_response_at_high_frequency() {
    let (mut circuit, output) = diode_varactor();
    circuit.set_device_parameter("v1", "ac", 1.0).expect("ac");
    circuit
        .set_device_parameter("vc", "ac", 0.0)
        .expect("cathode ac");
    let low = ac_analysis(&circuit, 1e3).expect("low");
    let high = ac_analysis(&circuit, 1e8).expect("high");
    let low = low.magnitude(output).expect("low magnitude");
    let high = high.magnitude(output).expect("high magnitude");
    assert!(low > 0.99, "low={low}");
    assert!(high < 0.5, "high={high}");
}

#[test]
fn mosfet_capacitance_reduces_gain_at_high_frequency() {
    let (mut circuit, _, output) = inverter();
    circuit
        .set_device_parameter("vin", "voltage", 2.5)
        .expect("bias");
    circuit.set_device_parameter("vin", "ac", 1.0).expect("ac");
    circuit
        .set_device_parameter("mp", "cox", 1e-12)
        .expect("mp");
    circuit
        .set_device_parameter("mn", "cox", 1e-12)
        .expect("mn");
    let low = ac_analysis(&circuit, 1e3).expect("low");
    let high = ac_analysis(&circuit, 1e9).expect("high");
    let low_gain = low.magnitude(output).expect("low gain");
    let high_gain = high.magnitude(output).expect("high gain");
    assert!(high_gain < low_gain, "low={low_gain} high={high_gain}");
}

#[test]
fn jfet_common_source_has_gain() {
    let (mut circuit, output) = common_source_n_biased();
    circuit.set_device_parameter("vg", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    assert!(magnitude > 0.1, "magnitude={magnitude}");
}

#[test]
fn bjt_common_emitter_has_gain() {
    let (mut circuit, output) = common_emitter();
    circuit.set_device_parameter("vb", "ac", 1.0).expect("ac");
    let result = ac_analysis(&circuit, 1000.0).expect("ac");
    let magnitude = result.magnitude(output).expect("magnitude");
    assert!(magnitude > 1.0, "magnitude={magnitude}");
}

#[test]
fn bjt_capacitance_reduces_gain_at_high_frequency() {
    let (mut circuit, output) = common_emitter();
    circuit.set_device_parameter("vb", "ac", 1.0).expect("ac");
    circuit
        .set_device_parameter("q1", "cje", 1e-9)
        .expect("cje");
    circuit
        .set_device_parameter("q1", "cjc", 1e-9)
        .expect("cjc");
    let low = ac_analysis(&circuit, 1e3).expect("low");
    let high = ac_analysis(&circuit, 1e9).expect("high");
    let low_gain = low.magnitude(output).expect("low gain");
    let high_gain = high.magnitude(output).expect("high gain");
    assert!(high_gain < low_gain, "low={low_gain} high={high_gain}");
}

#[test]
fn jfet_capacitance_reduces_gain_at_high_frequency() {
    let (mut circuit, output) = common_source_n_biased();
    circuit.set_device_parameter("vg", "ac", 1.0).expect("ac");
    circuit
        .set_device_parameter("j1", "cgs", 1e-9)
        .expect("cgs");
    circuit
        .set_device_parameter("j1", "cgd", 1e-9)
        .expect("cgd");
    let low = ac_analysis(&circuit, 1e3).expect("low");
    let high = ac_analysis(&circuit, 1e9).expect("high");
    let low_gain = low.magnitude(output).expect("low gain");
    let high_gain = high.magnitude(output).expect("high gain");
    assert!(high_gain < low_gain, "low={low_gain} high={high_gain}");
}
