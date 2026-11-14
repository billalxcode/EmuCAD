use super::*;

#[test]
fn set_wire_class_changes_existing_wire() {
    let mut document = Document::restore(
        Vec::new(),
        vec![test_wire(
            1,
            GridPos::new(0, 0),
            GridPos::new(2, 0),
            NetClass::Digital,
        )],
    );
    assert!(document.set_wire_class(WireId(1), NetClass::Power));
    assert_eq!(document.wires()[0].class, NetClass::Power);
    assert!(!document.set_wire_class(WireId(1), NetClass::Power));
    assert!(!document.set_wire_class(WireId(9), NetClass::Analog));
}

#[test]
fn net_wires_follow_shared_vertices() {
    let wires = vec![
        test_wire(1, GridPos::new(0, 0), GridPos::new(2, 0), NetClass::Digital),
        test_wire(2, GridPos::new(2, 0), GridPos::new(4, 0), NetClass::Digital),
        test_wire(3, GridPos::new(10, 0), GridPos::new(12, 0), NetClass::Power),
    ];
    let document = Document::restore(Vec::new(), wires);
    let net = document.net_wires(WireId(1));
    assert!(net.contains(&WireId(1)));
    assert!(net.contains(&WireId(2)));
    assert!(!net.contains(&WireId(3)));
}

#[test]
fn accepts_wire_between_pins() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let wire = document.add_wire(
        library,
        vec![GridPos::new(4, 1), GridPos::new(6, 1)],
        NetClass::Digital,
    );
    assert!(wire.is_some());
}

#[test]
fn rejects_wire_through_body() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let wire = document.add_wire(
        library,
        vec![GridPos::new(0, 1), GridPos::new(4, 1)],
        NetClass::Digital,
    );
    assert!(wire.is_none());
}

#[test]
fn rejects_diagonal_segment() {
    let document = Document::new();
    assert!(!document.can_add_segment(library::builtin(), GridPos::new(4, 1), GridPos::new(6, 2)));
}

#[test]
fn rejects_duplicate_edge() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let points = vec![GridPos::new(4, 1), GridPos::new(6, 1)];
    assert!(
        document
            .add_wire(library, points.clone(), NetClass::Digital)
            .is_some()
    );
    assert!(
        document
            .add_wire(library, points, NetClass::Digital)
            .is_none()
    );
}

#[test]
fn wire_hit_test_uses_tolerance() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let wire = document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        )
        .expect("wire");
    assert_eq!(document.wire_at([5.0, 1.2], 0.3), Some(wire));
    assert_eq!(document.wire_at([5.0, 2.0], 0.3), None);
}
