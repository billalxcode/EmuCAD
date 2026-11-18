use super::{Netlist, NetlistComponent, NetlistError, extract, solve_dc};
use crate::library;
use crate::model::{ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId};
use kernel::NodeId;

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

fn divider() -> Document {
    circuit("passive.resistor.ideal_resistor")
}

fn diode_clamp() -> Document {
    circuit("diode.rectifier.rectifier_diode")
}

fn circuit(second: &str) -> Document {
    let mut document = Document::default();
    add_circuit_parts(&mut document, second);
    add_divider_wires(&mut document);
    document
}

fn add_circuit_parts(document: &mut Document, second: &str) {
    add(document, "source.independent.dc_voltage_source", 0, 0);
    add(document, "passive.resistor.ideal_resistor", 8, 0);
    add(document, second, 8, 6);
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
fn changing_resistor_changes_divider() {
    let mut document = divider();
    set_value(&mut document, "R2", "2000 ohm");
    let netlist = extract(&document).expect("netlist");
    let result = solve_dc(&netlist).expect("dc");
    let output = find_part(&netlist, "R1").pins[1];
    let voltage = result
        .solution
        .voltage(node_id(output))
        .expect("output voltage");
    let expected = 5.0 * 2000.0 / 3000.0;
    assert!((voltage - expected).abs() < 1e-6, "voltage={voltage}");
}

#[test]
fn invalid_value_is_rejected() {
    let mut document = divider();
    set_value(&mut document, "R1", "not a number");
    assert!(matches!(
        extract(&document),
        Err(NetlistError::InvalidValue { .. })
    ));
}

fn set_value(document: &mut Document, label: &str, value: &str) {
    let id = component_id(document, label);
    document.component_mut(id).expect("component").value = value.to_string();
}

fn set_parameter(document: &mut Document, label: &str, name: &str, value: f64) {
    let id = component_id(document, label);
    document
        .component_mut(id)
        .expect("component")
        .parameters
        .insert(name.to_string(), value);
}

fn component_id(document: &Document, label: &str) -> ComponentId {
    document
        .components()
        .iter()
        .find(|part| part.label == label)
        .expect("component")
        .id
}

#[test]
fn diode_has_multiple_parameters() {
    let definition = library::find("diode.rectifier.rectifier_diode").expect("diode");
    let fields = super::parameter_fields(definition);
    let names: Vec<_> = fields.iter().map(|field| field.spec.name).collect();
    assert!(names.contains(&"is"));
    assert!(names.contains(&"n"));
    assert!(names.contains(&"rs"));
    assert!(
        fields
            .iter()
            .any(|field| field.primary && field.spec.name == "is")
    );
}

#[test]
fn diode_parameter_changes_clamp() {
    let mut document = diode_clamp();
    let before = clamp_voltage(&document);
    set_parameter(&mut document, "D1", "n", 2.0);
    let after = clamp_voltage(&document);
    assert!(
        (after - before).abs() > 1e-3,
        "before={before} after={after}"
    );
}

fn clamp_voltage(document: &Document) -> f64 {
    let netlist = extract(document).expect("netlist");
    let result = solve_dc(&netlist).expect("dc");
    let output = find_part(&netlist, "R1").pins[1];
    result
        .solution
        .voltage(node_id(output))
        .expect("output voltage")
}

#[test]
fn unknown_parameter_is_rejected() {
    let mut document = divider();
    set_parameter(&mut document, "R1", "bogus", 1.0);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::UnknownParameter { .. })
    ));
}

#[test]
fn out_of_range_parameter_is_rejected() {
    let mut document = divider();
    set_parameter(&mut document, "R1", "resistance", -1.0);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::ParameterOutOfRange { .. })
    ));
}

#[test]
fn numeric_values_are_parsed() {
    assert_eq!(super::parse_value("220"), Some(220.0));
    assert_eq!(super::parse_value("1000 ohm"), Some(1000.0));
    assert_eq!(super::parse_value("1e-6 F"), Some(1e-6));
    assert_eq!(super::parse_value("LED"), None);
}

fn bjt_circuit() -> Document {
    let mut document = Document::default();
    add_bjt_parts(&mut document);
    set_parameter(&mut document, "V2", "voltage", 0.8);
    add_bjt_wires(&mut document);
    document
}

fn add_bjt_parts(document: &mut Document) {
    add(document, "source.independent.dc_voltage_source", 0, 0);
    add(document, "passive.resistor.ideal_resistor", 8, 0);
    add(document, "transistor.bjt_npn.npn_small_signal", 8, 6);
    add(document, "source.independent.dc_voltage_source", 14, 10);
    add(document, "gnd", 8, 14);
}

fn add_bjt_wires(document: &mut Document) {
    wire(document, &[(0, 1), (0, -2), (8, -2), (8, 1)]);
    wire(document, &[(4, 1), (4, 14), (8, 14), (9, 14)]);
    wire(
        document,
        &[(12, 1), (14, 1), (14, 4), (6, 4), (6, 7), (8, 7)],
    );
    wire(document, &[(14, 11), (14, 8), (12, 8)]);
    wire(document, &[(8, 9), (8, 12), (8, 14)]);
    wire(document, &[(18, 11), (18, 12), (8, 12)]);
}

#[test]
fn bjt_common_emitter_saturates() {
    let document = bjt_circuit();
    let netlist = extract(&document).expect("netlist");
    let collector = find_part(&netlist, "Q1").pins[0];
    let base = find_part(&netlist, "Q1").pins[1];
    let result = solve_dc(&netlist).expect("dc");
    let collector_voltage = result
        .solution
        .voltage(node_id(collector))
        .expect("collector");
    let base_voltage = result.solution.voltage(node_id(base)).expect("base");
    assert!(collector_voltage < 1.0, "collector={collector_voltage}");
    assert!((base_voltage - 0.8).abs() < 0.1, "base={base_voltage}");
}

#[test]
fn diode_pin_roles_are_anode_and_cathode() {
    let definition = library::find("diode.rectifier.rectifier_diode").expect("diode");
    let roles = super::pin_roles(definition);
    assert_eq!(roles, &[device::PinRole::Anode, device::PinRole::Cathode]);
}

fn parallel_resistors() -> Document {
    let mut document = Document::default();
    add(&mut document, "source.independent.dc_voltage_source", 0, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 8, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 20, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 20, 6);
    add(&mut document, "gnd", 10, 20);
    wire(&mut document, &[(0, 1), (0, -4), (8, -4), (8, 1)]);
    wire(&mut document, &[(8, -4), (8, -8), (20, -8), (20, 1)]);
    wire(&mut document, &[(20, 1), (20, 7)]);
    wire(
        &mut document,
        &[(4, 1), (4, 22), (14, 22), (14, 20), (11, 20)],
    );
    wire(&mut document, &[(12, 1), (16, 1), (16, 22), (14, 22)]);
    wire(
        &mut document,
        &[(24, 1), (27, 1), (27, 10), (27, 22), (16, 22)],
    );
    wire(&mut document, &[(24, 7), (26, 7), (26, 10), (27, 10)]);
    document
}

#[test]
fn parallel_resistors_share_both_nets() {
    let netlist = extract(&parallel_resistors()).expect("netlist");
    let source = find_part(&netlist, "V1");
    let r1 = find_part(&netlist, "R1");
    let r2 = find_part(&netlist, "R2");
    let r3 = find_part(&netlist, "R3");
    assert_eq!(r1.pins[0], r2.pins[0]);
    assert_eq!(r2.pins[0], r3.pins[0]);
    assert_eq!(r1.pins[0], source.pins[0]);
    assert_eq!(r1.pins[1], r2.pins[1]);
    assert_eq!(r2.pins[1], r3.pins[1]);
    assert_eq!(r1.pins[1], netlist.ground);
}

#[test]
fn three_way_wire_junction_merges_nets() {
    let netlist = extract(&parallel_resistors()).expect("netlist");
    let r1 = find_part(&netlist, "R1");
    let r2 = find_part(&netlist, "R2");
    assert_eq!(r1.pins[0], r2.pins[0]);
}

#[test]
fn duplicate_labels_do_not_conflict() {
    let mut document = parallel_resistors();
    let first = document.wires()[0].id;
    let second = document.wires()[1].id;
    document.set_wire_label(first, Some("SIG".to_string()));
    document.set_wire_label(second, Some("SIG".to_string()));
    let netlist = extract(&document).expect("netlist");
    assert!(netlist.nodes.iter().any(|node| node.name == "SIG"));
}

#[test]
fn conflicting_labels_are_rejected() {
    let mut document = parallel_resistors();
    let first = document.wires()[0].id;
    let second = document.wires()[1].id;
    document.set_wire_label(first, Some("A".to_string()));
    document.set_wire_label(second, Some("B".to_string()));
    assert!(matches!(
        extract(&document),
        Err(NetlistError::LabelConflict { .. })
    ));
}

#[test]
fn overlapping_pins_are_rejected() {
    let mut document = Document::default();
    add(&mut document, "passive.resistor.ideal_resistor", 0, 0);
    add(&mut document, "passive.resistor.ideal_resistor", 0, 0);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::OverlappingPins { .. })
    ));
}

#[test]
fn dangling_wire_endpoint_is_rejected() {
    let mut document = Document::default();
    add(&mut document, "passive.resistor.ideal_resistor", 0, 0);
    let ground = add(&mut document, "gnd", 0, 2);
    wire(&mut document, &[(0, 1), (0, 2), (1, 2)]);
    document.remove(ground);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::WireStub { .. })
    ));
}

#[test]
fn undriven_island_is_rejected() {
    let mut document = parallel_resistors();
    add(&mut document, "passive.resistor.ideal_resistor", 34, 0);
    wire(&mut document, &[(34, 1), (34, -2), (38, -2), (38, 1)]);
    assert!(matches!(
        extract(&document),
        Err(NetlistError::FloatingIsland { .. })
    ));
}

#[test]
fn fidelity_labels_follow_catalog_classes() {
    let resistor = library::find("passive.resistor.ideal_resistor").expect("resistor");
    let diode = library::find("diode.rectifier.rectifier_diode").expect("diode");
    assert_eq!(super::fidelity_label(resistor), Some("ideal"));
    assert_eq!(super::fidelity_label(diode), Some("compact"));
}
