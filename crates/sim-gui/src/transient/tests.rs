use super::{TransientSettings, TransientTrace, solve};
use crate::library;
use crate::model::{ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId};
use crate::netlist;

fn add(document: &mut Document, id: &str, x: i32, y: i32) -> ComponentId {
    let definition = library::find(id).expect("definition");
    document.add(definition, GridPos::new(x, y), Rotation::Deg0, Mirror::None)
}

fn wire(document: &mut Document, points: &[(i32, i32)]) -> WireId {
    let points: Vec<_> = points.iter().map(|&(x, y)| GridPos::new(x, y)).collect();
    let id = document.add_wire(library::builtin(), points.clone(), NetClass::Power);
    assert!(id.is_some(), "wire {points:?}");
    id.expect("wire")
}

fn circuit(second: &str) -> Document {
    let mut document = Document::default();
    add_parts(&mut document, second);
    set_pulse_defaults(&mut document);
    add_wires(&mut document);
    document
}

fn add_parts(document: &mut Document, second: &str) {
    add(document, "source.pulse.pulse_voltage_source", 0, 0);
    add(document, "passive.resistor.ideal_resistor", 8, 0);
    add(document, second, 8, 6);
    add(document, "gnd", 10, 10);
}

fn set_pulse_defaults(document: &mut Document) {
    set_parameter(document, "V1", "width", 1.0);
    set_parameter(document, "V1", "period", 2.0);
    set_parameter(document, "V1", "rise", 1e-6);
    set_parameter(document, "V1", "fall", 1e-6);
}

fn add_wires(document: &mut Document) {
    wire(document, &[(0, 1), (0, -2), (8, -2), (8, 1)]);
    wire(document, &[(4, 1), (4, 10), (11, 10)]);
    wire(document, &[(12, 1), (12, 5), (6, 5), (6, 7), (8, 7)]);
    wire(document, &[(12, 7), (12, 10), (11, 10)]);
}

fn rc_circuit(capacitance: f64) -> Document {
    let mut document = circuit("passive.capacitor.ideal_capacitor");
    set_parameter(&mut document, "C1", "capacitance", capacitance);
    document
}

fn rl_circuit(inductance: f64) -> Document {
    let mut document = circuit("passive.inductor.ideal_inductor");
    set_parameter(&mut document, "R1", "resistance", 100.0);
    set_parameter(&mut document, "L1", "inductance", inductance);
    document
}

fn diode_circuit(junction_capacitance: f64) -> Document {
    let mut document = circuit("diode.rectifier.rectifier_diode");
    set_parameter(&mut document, "D1", "cj0", junction_capacitance);
    document
}

fn pulse_resistor_circuit(width: f64) -> Document {
    let mut document = circuit("passive.resistor.ideal_resistor");
    set_parameter(&mut document, "V1", "width", width);
    document
}

fn pulse_rise_circuit(rise: f64) -> Document {
    let mut document = pulse_resistor_circuit(1.0);
    set_parameter(&mut document, "V1", "rise", rise);
    document
}

fn set_parameter(document: &mut Document, label: &str, name: &str, value: f64) {
    let id = document
        .components()
        .iter()
        .find(|part| part.label == label)
        .expect("component")
        .id;
    document
        .component_mut(id)
        .expect("component")
        .parameters
        .insert(name.to_string(), value);
}

fn output_node(document: &Document) -> usize {
    node_of(document, "R1", 1)
}

fn node_of(document: &Document, label: &str, pin: usize) -> usize {
    let netlist = netlist::extract(document).expect("netlist");
    let component = netlist
        .components
        .iter()
        .find(|part| part.label == label)
        .expect("component");
    component.pins[pin]
}

fn value_at(trace: &TransientTrace, node: usize, time: f64) -> f64 {
    let index = trace.index_at(time).expect("time");
    trace.values[index][node]
}

fn rectifier_circuit() -> Document {
    let mut document = Document::default();
    add(&mut document, "source.pulse.pulse_voltage_source", 0, 0);
    add(&mut document, "diode.rectifier.rectifier_diode", 8, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 8, 6);
    add(&mut document, "gnd", 10, 10);
    set_pulse_defaults(&mut document);
    add_wires(&mut document);
    document
}

fn line_circuit() -> Document {
    let mut document = Document::default();
    add_line_parts(&mut document);
    set_line_parameters(&mut document);
    add_line_wires(&mut document);
    document
}

fn add_line_parts(document: &mut Document) {
    add(document, "source.pulse.pulse_voltage_source", 0, 0);
    add(
        document,
        "passive.transmission_line.ideal_transmission_line",
        8,
        0,
    );
    add(document, "passive.resistor.ideal_resistor", 8, 8);
    add(document, "gnd", 13, 12);
}

fn set_line_parameters(document: &mut Document) {
    set_pulse_defaults(document);
    set_parameter(document, "R1", "resistance", 50.0);
    let line = document
        .components()
        .iter()
        .find(|part| part.definition.contains("transmission_line"))
        .expect("line")
        .id;
    document
        .component_mut(line)
        .expect("line")
        .parameters
        .insert("delay".to_string(), 1e-5);
}

fn add_line_wires(document: &mut Document) {
    wire(document, &[(0, 1), (0, -2), (8, -2), (8, 1)]);
    wire(document, &[(4, 1), (4, 12), (13, 12), (14, 12)]);
    wire(
        document,
        &[
            (12, 1),
            (14, 1),
            (14, 3),
            (14, 6),
            (18, 6),
            (18, 12),
            (14, 12),
        ],
    );
    wire(document, &[(12, 3), (14, 3)]);
    wire(document, &[(8, 3), (8, 9)]);
    wire(document, &[(12, 9), (13, 9), (13, 12)]);
}

fn device_index(trace: &TransientTrace, label: &str) -> usize {
    trace
        .devices
        .iter()
        .position(|device| device.label == label)
        .expect("device")
}

fn current_at(trace: &TransientTrace, label: &str, time: f64) -> f64 {
    let index = trace.index_at(time).expect("time");
    let device = device_index(trace, label);
    trace.currents[index][device].expect("current")
}

fn transient_settings(dt: f64, steps: usize) -> TransientSettings {
    TransientSettings {
        dt,
        steps,
        use_dc_op: true,
    }
}

#[test]
fn rc_capacitor_charges_toward_supply() {
    let document = rc_circuit(1e-6);
    let output = output_node(&document);
    let trace = solve(&document, &TransientSettings::default()).expect("trace");
    let last = trace.values.last().expect("last sample");
    assert!(last[output] > 4.9, "voltage={}", last[output]);
}

#[test]
fn larger_capacitance_charges_slower() {
    let fast = rc_circuit(1e-6);
    let slow = rc_circuit(2e-6);
    let fast_node = output_node(&fast);
    let slow_node = output_node(&slow);
    let settings = TransientSettings::default();
    let fast_trace = solve(&fast, &settings).expect("fast");
    let slow_trace = solve(&slow, &settings).expect("slow");
    let index = fast_trace.index_at(2e-3).expect("time");
    assert!(fast_trace.values[index][fast_node] > slow_trace.values[index][slow_node] + 0.1);
}

#[test]
fn larger_inductance_decays_slower() {
    let small = rl_circuit(1e-3);
    let large = rl_circuit(2e-3);
    let small_node = output_node(&small);
    let large_node = output_node(&large);
    let settings = transient_settings(1e-6, 100);
    let small_trace = solve(&small, &settings).expect("small");
    let large_trace = solve(&large, &settings).expect("large");
    let index = small_trace.index_at(20e-6).expect("time");
    assert!(
        large_trace.values[index][large_node] > small_trace.values[index][small_node] + 0.1,
        "nodes={:?} small={} large={}",
        small_trace.nodes,
        small_trace.values[index][small_node],
        large_trace.values[index][large_node]
    );
}

#[test]
fn junction_capacitance_slows_diode_turn_on() {
    let fast = diode_circuit(0.0);
    let slow = diode_circuit(1e-6);
    let fast_node = output_node(&fast);
    let slow_node = output_node(&slow);
    let settings = transient_settings(1e-5, 100);
    let fast_trace = solve(&fast, &settings).expect("fast");
    let slow_trace = solve(&slow, &settings).expect("slow");
    let index = fast_trace.index_at(100e-6).expect("time");
    assert!(fast_trace.values[index][fast_node] > slow_trace.values[index][slow_node] + 0.05);
}

#[test]
fn pulse_width_changes_high_time() {
    let long = pulse_resistor_circuit(0.5);
    let short = pulse_resistor_circuit(0.1);
    let long_node = output_node(&long);
    let short_node = output_node(&short);
    let settings = transient_settings(1e-3, 1000);
    let long_trace = solve(&long, &settings).expect("long");
    let short_trace = solve(&short, &settings).expect("short");
    let index = long_trace.index_at(0.3).expect("time");
    assert!(long_trace.values[index][long_node] > 1.0);
    assert!(short_trace.values[index][short_node] < 0.1);
}

#[test]
fn pulse_rise_changes_edge_speed() {
    let fast = pulse_rise_circuit(1e-6);
    let slow = pulse_rise_circuit(1e-3);
    let fast_node = output_node(&fast);
    let slow_node = output_node(&slow);
    let settings = transient_settings(1e-5, 100);
    let fast_trace = solve(&fast, &settings).expect("fast");
    let slow_trace = solve(&slow, &settings).expect("slow");
    let index = fast_trace.index_at(0.5e-3).expect("time");
    assert!(fast_trace.values[index][fast_node] > slow_trace.values[index][slow_node] + 0.5);
}

#[test]
fn resistor_current_matches_ohm_law() {
    let document = pulse_resistor_circuit(0.5);
    let trace = solve(&document, &transient_settings(1e-3, 1000)).expect("trace");
    let resistor = current_at(&trace, "R1", 0.3);
    assert!((resistor - 2.5e-3).abs() < 1e-5, "current={resistor}");
    let source = current_at(&trace, "V1", 0.3);
    assert!((source.abs() - 2.5e-3).abs() < 1e-5, "current={source}");
}

#[test]
fn inductor_current_rises_slower_with_larger_l() {
    let small = rl_circuit(1e-3);
    let large = rl_circuit(2e-3);
    let settings = transient_settings(1e-6, 100);
    let small_trace = solve(&small, &settings).expect("small");
    let large_trace = solve(&large, &settings).expect("large");
    let small_current = current_at(&small_trace, "L1", 20e-6);
    let large_current = current_at(&large_trace, "L1", 20e-6);
    assert!(
        (small_current - 0.043).abs() < 0.01,
        "current={small_current}"
    );
    assert!(small_current > large_current + 0.005);
}

#[test]
fn capacitor_current_is_positive_while_charging() {
    let document = rc_circuit(1e-6);
    let trace = solve(&document, &TransientSettings::default()).expect("trace");
    let current = current_at(&trace, "C1", 1e-3);
    assert!(current > 1e-3 && current < 3e-3, "current={current}");
}

#[test]
fn diode_current_is_forward() {
    let document = diode_circuit(0.0);
    let trace = solve(&document, &transient_settings(1e-5, 100)).expect("trace");
    let current = current_at(&trace, "D1", 1e-3);
    assert!(current > 1e-3, "current={current}");
}

#[test]
fn diode_rectifier_passes_positive_half() {
    let document = rectifier_circuit();
    let output = node_of(&document, "R1", 0);
    let trace = solve(&document, &transient_settings(1e-3, 2000)).expect("trace");
    let high = value_at(&trace, output, 0.5);
    let low = value_at(&trace, output, 1.5);
    assert!(high > 4.0, "high={high}");
    assert!(low < 0.1, "low={low}");
}

#[test]
fn transmission_line_delays_pulse() {
    let document = line_circuit();
    let output = node_of(&document, "R1", 0);
    let settings = TransientSettings {
        dt: 1e-6,
        steps: 100,
        use_dc_op: false,
    };
    let trace = solve(&document, &settings).expect("trace");
    let before = value_at(&trace, output, 5e-6);
    let after = value_at(&trace, output, 15e-6);
    assert!(before < 0.5, "before={before}");
    assert!(after > 2.0, "after={after}");
}

#[test]
fn invalid_settings_are_rejected() {
    let document = rc_circuit(1e-6);
    let zero_dt = TransientSettings {
        dt: 0.0,
        ..TransientSettings::default()
    };
    assert!(solve(&document, &zero_dt).is_err());
    let too_long = TransientSettings {
        dt: 1.0,
        steps: 11,
        use_dc_op: true,
    };
    assert!(solve(&document, &too_long).is_err());
}
