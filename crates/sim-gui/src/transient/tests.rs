use super::{TransientSettings, solve};
use crate::library;
use crate::model::{ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId};
use crate::netlist;

fn add(document: &mut Document, id: &str, x: i32, y: i32) -> ComponentId {
    let definition = library::find(id).expect("definition");
    document.add(definition, GridPos::new(x, y), Rotation::Deg0, Mirror::None)
}

fn wire(document: &mut Document, points: &[(i32, i32)]) -> WireId {
    let points = points.iter().map(|&(x, y)| GridPos::new(x, y)).collect();
    document
        .add_wire(library::builtin(), points, NetClass::Power)
        .expect("wire")
}

fn rc_circuit(capacitance: f64) -> Document {
    let mut document = Document::default();
    add(&mut document, "source.pulse.pulse_voltage_source", 0, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 8, 0);
    add(&mut document, "passive.capacitor.ideal_capacitor", 8, 6);
    add(&mut document, "gnd", 10, 10);
    set_parameter(&mut document, "V1", "width", 1.0);
    set_parameter(&mut document, "V1", "period", 2.0);
    set_parameter(&mut document, "V1", "rise", 1e-6);
    set_parameter(&mut document, "V1", "fall", 1e-6);
    set_parameter(&mut document, "C1", "capacitance", capacitance);
    wire(&mut document, &[(0, 1), (0, -2), (8, -2), (8, 1)]);
    wire(&mut document, &[(4, 1), (4, 10), (11, 10)]);
    wire(&mut document, &[(12, 1), (12, 5), (6, 5), (6, 7), (8, 7)]);
    wire(&mut document, &[(12, 7), (12, 10), (11, 10)]);
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
    let netlist = netlist::extract(document).expect("netlist");
    let resistor = netlist
        .components
        .iter()
        .find(|part| part.label == "R1")
        .expect("R1");
    resistor.pins[1]
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
