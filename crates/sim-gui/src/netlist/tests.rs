use super::{Netlist, NetlistComponent, NetlistError, extract, solve_dc};
use crate::library;
use crate::model::{ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId};
use sim_core::NodeId;

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

fn divider() -> Document {
    let mut document = Document::default();
    add_divider_parts(&mut document);
    add_divider_wires(&mut document);
    document
}

fn add_divider_parts(document: &mut Document) {
    add(document, "source.independent.dc_voltage_source", 0, 0);
    add(document, "passive.resistor.ideal_resistor", 8, 0);
    add(document, "passive.resistor.ideal_resistor", 8, 6);
    add(document, "gnd", 10, 10);
    add(document, "vcc", 0, 12);
}

fn add_divider_wires(document: &mut Document) {
    wire(document, &[(0, 1), (0, 14), (1, 14)]);
    wire(document, &[(4, 1), (4, 10), (11, 10)]);
    wire(document, &[(8, 1), (8, -2), (0, -2), (0, 1)]);
    wire(document, &[(12, 1), (12, 5), (6, 5), (6, 7), (8, 7)]);
    wire(document, &[(12, 7), (12, 10), (11, 10)]);
}

fn find_part<'a>(netlist: &'a Netlist, label: &str) -> &'a NetlistComponent {
    netlist
        .components
        .iter()
        .find(|part| part.label == label)
        .expect("component")
}

#[test]
fn extracts_resistor_divider() {
    let netlist = extract(&divider()).expect("netlist");
    assert_eq!(netlist.ground, 0);
    assert_eq!(netlist.nodes[0].name, "0");
    assert_eq!(netlist.components.len(), 3);
    assert_voltage_divider(&netlist);
}

fn assert_voltage_divider(netlist: &Netlist) {
    let r1 = find_part(netlist, "R1");
    let r2 = find_part(netlist, "R2");
    let source = find_part(netlist, "V1");
    assert_eq!(r1.pins[1], r2.pins[0]);
    assert_ne!(r1.pins[0], r1.pins[1]);
    assert_eq!(r2.pins[1], netlist.ground);
    assert_eq!(source.pins[1], netlist.ground);
    assert_eq!(r1.parameters.get("resistance"), Some(&1000.0));
    assert!(netlist.nodes.iter().any(|node| node.name == "VCC"));
}

#[test]
fn missing_ground_is_rejected() {
    let mut document = Document::default();
    add(&mut document, "passive.resistor.ideal_resistor", 0, 0);
    add(&mut document, "vcc", 0, 4);
    wire(&mut document, &[(0, 1), (0, 6), (1, 6)]);
    assert_eq!(extract(&document), Err(NetlistError::MissingGround));
}

#[test]
fn multiple_grounds_are_rejected() {
    let mut document = Document::default();
    add(&mut document, "gnd", 0, 0);
    add(&mut document, "gnd", 10, 0);
    assert_eq!(extract(&document), Err(NetlistError::MultipleGrounds(2)));
}

#[test]
fn unsupported_component_is_rejected() {
    let mut document = Document::default();
    add(&mut document, "atmega328p", 0, 0);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::Unsupported(_))
    ));
}

#[test]
fn floating_pin_is_rejected() {
    let mut document = Document::default();
    add(&mut document, "passive.resistor.ideal_resistor", 0, 0);
    add(&mut document, "gnd", 0, 2);
    wire(&mut document, &[(0, 1), (0, 2), (1, 2)]);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::FloatingPin { .. })
    ));
}

#[test]
fn solves_resistor_divider() {
    let netlist = extract(&divider()).expect("netlist");
    let result = solve_dc(&netlist).expect("dc");
    let output = find_part(&netlist, "R1").pins[1];
    let voltage = result
        .solution
        .voltage(node_id(output))
        .expect("output voltage");
    assert!((voltage - 2.5).abs() < 1e-6, "voltage={voltage}");
}

fn node_id(index: usize) -> NodeId {
    NodeId::new(u32::try_from(index).expect("node index"))
}

#[test]
fn numeric_values_are_parsed() {
    assert_eq!(super::parse_value("220"), Some(220.0));
    assert_eq!(super::parse_value("1000 ohm"), Some(1000.0));
    assert_eq!(super::parse_value("1e-6 F"), Some(1e-6));
    assert_eq!(super::parse_value("LED"), None);
}
