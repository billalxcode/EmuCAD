use super::*;

#[test]
fn rotation_swaps_body_size() {
    assert_eq!(body_size(&RESISTOR, Rotation::Deg0).width, 4);
    assert_eq!(body_size(&RESISTOR, Rotation::Deg90).width, 2);
    assert_eq!(body_size(&RESISTOR, Rotation::Deg90).height, 4);
}

#[test]
fn pins_stay_on_grid_after_transform() {
    for rotation in [
        Rotation::Deg0,
        Rotation::Deg90,
        Rotation::Deg180,
        Rotation::Deg270,
    ] {
        for mirror in [Mirror::None, Mirror::Horizontal, Mirror::Vertical] {
            let mut document = Document::new();
            let id = document.add(&RESISTOR, GridPos::new(10, 10), rotation, mirror);
            let instance = document.component(id).expect("instance");
            for pin in PINS {
                let position = world_pin(instance, &RESISTOR, pin);
                assert!(position.x >= 10 && position.x <= 14);
                assert!(position.y >= 10 && position.y <= 14);
            }
        }
    }
}

#[test]
fn overlap_rejects_interior_and_allows_touching_edge() {
    let mut document = Document::new();
    document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    assert!(!document.can_place(
        LIBRARY,
        &RESISTOR,
        GridPos::new(3, 0),
        Rotation::Deg0,
        Mirror::None
    ));
    assert!(document.can_place(
        LIBRARY,
        &RESISTOR,
        GridPos::new(4, 0),
        Rotation::Deg0,
        Mirror::None
    ));
}

#[test]
fn selection_rect_finds_components_and_wires() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let wire = document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        )
        .expect("wire");
    let rect = super::GridRect::from_corners(GridPos::new(-1, -1), GridPos::new(10, 3));
    assert_eq!(
        document.components_in_rect(library, rect),
        vec![first, second]
    );
    assert_eq!(document.wires_in_rect(rect), vec![wire]);
}

#[test]
fn selection_rect_requires_full_containment() {
    let mut document = Document::new();
    let id = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let partial = super::GridRect::from_corners(GridPos::new(1, 0), GridPos::new(3, 2));
    assert!(document.components_in_rect(LIBRARY, partial).is_empty());
    let full = super::GridRect::from_corners(GridPos::new(0, 0), GridPos::new(4, 2));
    assert_eq!(document.components_in_rect(LIBRARY, full), vec![id]);
}

#[test]
fn rejects_crossing_without_junction() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(10, 0), Rotation::Deg0, Mirror::None);
    document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(10, 1)],
            NetClass::Digital,
        )
        .expect("wire");
    assert!(!document.can_add_segment(library, GridPos::new(7, 0), GridPos::new(7, 2)));
    assert!(document.can_add_segment(library, GridPos::new(4, 1), GridPos::new(4, 3)));
}

#[test]
fn detects_component_wires() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let third = document.add(resistor, GridPos::new(20, 0), Rotation::Deg0, Mirror::None);
    document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        )
        .expect("wire");
    assert!(document.component_has_wire(library, first));
    assert!(document.component_has_wire(library, second));
    assert!(!document.component_has_wire(library, third));
}

#[test]
fn pin_wire_lookup_finds_connected_wire() {
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
    assert_eq!(
        document.pin_wire_at(library, GridPos::new(4, 1)),
        Some(wire)
    );
    assert_eq!(document.pin_wire_at(library, GridPos::new(1, 1)), None);
}

#[test]
fn snap_pin_maps_visible_dot() {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    assert_eq!(
        document.snap_pin(library, GridPos::new(5, 1)),
        Some(GridPos::new(4, 1))
    );
}
