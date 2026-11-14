use super::*;

#[test]
fn update_wire_points_moves_segment() {
    let library = library::builtin();
    let (mut document, wire) = segment_document();
    let moved = vec![
        GridPos::new(4, 1),
        GridPos::new(4, 2),
        GridPos::new(6, 2),
        GridPos::new(6, 1),
    ];
    assert!(document.update_wire_points(library, wire, moved.clone()));
    assert_eq!(document.wires()[0].points, moved);
    let invalid = vec![
        GridPos::new(4, 1),
        GridPos::new(4, 1),
        GridPos::new(6, 1),
        GridPos::new(6, 1),
    ];
    assert!(!document.update_wire_points(library, wire, invalid));
}

fn junction_document() -> Document {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let vcc = library::find("vcc").expect("vcc");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    document.add(vcc, GridPos::new(4, -2), Rotation::Deg0, Mirror::None);
    add_junction_wires(&mut document, library);
    document
}

fn add_junction_wires(document: &mut Document, library: &[super::ComponentDefinition]) {
    let horizontal = vec![GridPos::new(4, 1), GridPos::new(5, 1), GridPos::new(6, 1)];
    let vertical = vec![GridPos::new(5, 1), GridPos::new(5, 0)];
    assert!(
        document
            .add_wire(library, horizontal, NetClass::Digital)
            .is_some()
    );
    assert!(
        document
            .add_wire(library, vertical, NetClass::Power)
            .is_some()
    );
}

#[test]
fn junction_detected_at_three_segments() {
    let document = junction_document();
    assert_eq!(document.junctions(), vec![GridPos::new(5, 1)]);
}
