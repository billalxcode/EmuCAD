use super::{
    TransientConfig, TransientSample, transient, transient_with_method, transient_with_solver,
};
use crate::newton::NewtonConfig;
use crate::test_support::{
    common_emitter, common_source_n_biased, diode_rectifier, matched_line, mos_capacitor,
    pulse_divider, rc_circuit, rc_circuit_no_source, reverse_diode_rc, rl_circuit,
    switching_inverter,
};
use sim_core::NodeId;
use sim_device::IntegrationMethod;

#[test]
fn rc_charges_toward_supply() {
    let (circuit, output) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 1000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let last = samples.last().expect("last");
    let voltage = last.solution.voltage(output).expect("output");
    assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
}

#[test]
fn transient_rejects_zero_dt() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: 0.0,
        steps: 1,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    assert!(transient(&circuit, &config).is_err());
}

#[test]
fn transient_rejects_negative_dt() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: -1e-6,
        steps: 1,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    assert!(transient(&circuit, &config).is_err());
}

#[test]
fn transient_rejects_negative_gmin() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 1,
        gmin: -1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    assert!(transient(&circuit, &config).is_err());
}

#[test]
fn transient_time_increases() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 2,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    assert_eq!(samples[0].time.as_ps(), 1_000_000);
    assert_eq!(samples[1].time.as_ps(), 2_000_000);
}

#[test]
fn transient_zero_steps_is_empty() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 0,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    assert!(transient(&circuit, &config).expect("transient").is_empty());
}

#[test]
fn transient_rejects_huge_dt() {
    let (circuit, _) = rc_circuit();
    let config = TransientConfig {
        dt: 1e20,
        steps: 1,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    assert!(transient(&circuit, &config).is_err());
}

#[test]
fn dc_initial_condition_is_steady() {
    let (circuit, output) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 5,
        gmin: 1e-12,
        use_dc_op: true,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let last = samples.last().expect("last");
    let voltage = last.solution.voltage(output).expect("output");
    assert!((voltage - 5.0).abs() < 1e-6);
}

#[test]
fn trapezoidal_rc_charges_toward_supply() {
    let (circuit, output) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 1000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient_with_method(&circuit, &config, IntegrationMethod::Trapezoidal)
        .expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
}

#[test]
fn gear2_rc_charges_toward_supply() {
    let (circuit, output) = rc_circuit();
    let config = TransientConfig {
        dt: 1e-6,
        steps: 1000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples =
        transient_with_method(&circuit, &config, IntegrationMethod::Gear2).expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!((voltage - 3.16).abs() < 0.15, "voltage={voltage}");
}

#[test]
fn trapezoidal_rl_voltage_decays() {
    let (circuit, output) = rl_circuit();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 1000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient_with_method(&circuit, &config, IntegrationMethod::Trapezoidal)
        .expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!(voltage.abs() < 0.05, "voltage={voltage}");
}

#[test]
fn gear2_rl_voltage_decays() {
    let (circuit, output) = rl_circuit();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 1000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples =
        transient_with_method(&circuit, &config, IntegrationMethod::Gear2).expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!(voltage.abs() < 0.05, "voltage={voltage}");
}

#[test]
fn transmission_line_delays_matched_step() {
    let (circuit, output) = matched_line();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 20,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let early = samples[8].solution.voltage(output).expect("early");
    let late = samples[14].solution.voltage(output).expect("late");
    assert!(early.abs() < 0.01, "early={early}");
    assert!((late - 2.5).abs() < 0.1, "late={late}");
}

#[test]
fn rc_discharge_decays_from_initial_voltage() {
    let (circuit, output) = rc_circuit_no_source();
    let config = TransientConfig {
        dt: 1e-4,
        steps: 100,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: vec![(output, 5.0)],
    };
    let samples = transient(&circuit, &config).expect("transient");
    let early = samples[0].solution.voltage(output).expect("early");
    let last = samples.last().expect("last");
    let last = last.solution.voltage(output).expect("last");
    assert!(early > 4.0, "early={early}");
    assert!(last.abs() < 0.01, "last={last}");
}

#[test]
fn pulse_source_switches_transient_output() {
    let (circuit, output) = pulse_divider();
    let config = TransientConfig {
        dt: 0.25e-6,
        steps: 8,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let high = samples[0].solution.voltage(output).expect("high");
    let low = samples[5].solution.voltage(output).expect("low");
    let again = samples[7].solution.voltage(output).expect("again");
    assert!((high - 2.5).abs() < 1e-6, "high={high}");
    assert!(low.abs() < 1e-6, "low={low}");
    assert!((again - 2.5).abs() < 1e-6, "again={again}");
}

#[test]
fn mosfet_gate_capacitance_charges() {
    let (circuit, output) = mos_capacitor();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 20,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let gate = samples.last().expect("last");
    let gate = gate.solution.voltage(output).expect("gate");
    assert!(gate > 2.8 && gate < 3.3, "gate={gate}");
}

#[test]
fn diode_junction_capacitance_slows_reverse_edge() {
    let config = TransientConfig {
        dt: 1e-7,
        steps: 10,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let (without, without_output) = reverse_diode_rc(0.0);
    let (with, with_output) = reverse_diode_rc(5e-9);
    let without = transient(&without, &config).expect("without");
    let with = transient(&with, &config).expect("with");
    let without = without.last().expect("last");
    let with = with.last().expect("last");
    let without = without.solution.voltage(without_output).expect("v0");
    let with = with.solution.voltage(with_output).expect("v1");
    assert!(without < -1.9, "without={without}");
    assert!(with > -1.5, "with={with}");
}

#[test]
fn diode_rectifier_transient_reaches_forward_drop() {
    let (circuit, output) = diode_rectifier();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 50,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!(voltage > 0.4 && voltage < 0.9, "voltage={voltage}");
}

#[test]
fn switching_inverter_transient_crosses_output() {
    let (circuit, output) = switching_inverter();
    let config = TransientConfig {
        dt: 1e-9,
        steps: 4000,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    assert!(count_crossings(&samples, output) >= 2);
}

#[test]
fn bjt_common_emitter_transient_reaches_bias() {
    let (circuit, output) = common_emitter();
    let config = TransientConfig {
        dt: 1e-7,
        steps: 20,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let samples = transient(&circuit, &config).expect("transient");
    let voltage = samples
        .last()
        .expect("last")
        .solution
        .voltage(output)
        .expect("v");
    assert!(voltage > 3.0 && voltage < 4.5, "voltage={voltage}");
}

#[test]
fn bjt_capacitance_slows_base_charge() {
    let (without, without_output) = common_emitter();
    let (mut with, with_output) = common_emitter();
    with.set_device_parameter("q1", "cje", 1e-8).expect("cje");
    let config = TransientConfig {
        dt: 1e-5,
        steps: 20,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let without = final_voltage(&without, &config, without_output);
    let with = final_voltage(&with, &config, with_output);
    assert!(with > without + 0.1, "without={without} with={with}");
}

#[test]
fn jfet_capacitance_slows_drain_rise() {
    let (without, without_output) = common_source_n_biased();
    let (mut with, with_output) = common_source_n_biased();
    with.set_device_parameter("j1", "cgd", 1e-8).expect("cgd");
    let config = TransientConfig {
        dt: 1e-6,
        steps: 10,
        gmin: 1e-12,
        use_dc_op: false,
        initial_voltages: Vec::new(),
    };
    let without = final_voltage(&without, &config, without_output);
    let with = final_voltage(&with, &config, with_output);
    assert!(with < without, "without={without} with={with}");
}

#[test]
fn transient_rejects_invalid_newton_config() {
    let (circuit, _) = rc_circuit();
    let newton = NewtonConfig {
        max_iterations: 0,
        ..NewtonConfig::default()
    };
    let method = IntegrationMethod::BackwardEuler;
    let config = TransientConfig::default();
    assert!(transient_with_solver(&circuit, &config, method, &newton).is_err());
}

fn final_voltage(circuit: &sim_core::Circuit, config: &TransientConfig, node: NodeId) -> f64 {
    let samples = transient(circuit, config).expect("transient");
    samples
        .last()
        .expect("last")
        .solution
        .voltage(node)
        .expect("voltage")
}

fn count_crossings(samples: &[TransientSample], node: NodeId) -> usize {
    let mut previous = samples[0].solution.voltage(node).expect("first") > 2.5;
    let mut crossings = 0;
    for sample in samples.iter().skip(1) {
        let high = sample.solution.voltage(node).expect("voltage") > 2.5;
        if high != previous {
            crossings += 1;
            previous = high;
        }
    }
    crossings
}
