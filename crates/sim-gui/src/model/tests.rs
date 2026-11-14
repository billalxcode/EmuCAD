use super::{
    ComponentDefinition, Document, GridPos, GridSize, Mirror, NetClass, PinDefinition, Rotation,
    SymbolKind, Wire, WireId, body_size, world_pin,
};
use crate::library;

const PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "1",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "2",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const RESISTOR: ComponentDefinition = ComponentDefinition {
    id: "resistor",
    name: "Resistor",
    category: super::Category::Passive,
    reference_prefix: "R",
    default_value: "220",
    size: GridSize::new(4, 2),
    pins: PINS,
    symbol: SymbolKind::Resistor,
};

const LIBRARY: &[ComponentDefinition] = &[RESISTOR];

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

fn test_wire(id: u32, start: GridPos, end: GridPos, class: NetClass) -> Wire {
    Wire {
        id: WireId(id),
        points: vec![start, end],
        class,
        color_override: None,
        label: None,
    }
}

#[test]
fn move_components_is_atomic() {
    let mut document = Document::new();
    let first = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(&RESISTOR, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    assert!(document.move_components(
        LIBRARY,
        &[(first, GridPos::new(0, 4)), (second, GridPos::new(6, 4))]
    ));
    assert_eq!(
        document.component(first).expect("first").position,
        GridPos::new(0, 4)
    );
    assert!(!document.move_components(LIBRARY, &[(first, GridPos::new(6, 4))]));
    assert_eq!(
        document.component(first).expect("first").position,
        GridPos::new(0, 4)
    );
}

#[test]
fn rotate_and_mirror_apply_to_group() {
    let mut document = Document::new();
    let first = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(&RESISTOR, GridPos::new(5, 0), Rotation::Deg0, Mirror::None);
    assert!(document.rotate_components(LIBRARY, &[first, second]));
    assert_eq!(
        document.component(first).expect("first").rotation,
        Rotation::Deg90
    );
    assert_eq!(
        document.component(second).expect("second").rotation,
        Rotation::Deg90
    );
    assert!(document.mirror_components(LIBRARY, &[first, second]));
    assert_eq!(
        document.component(first).expect("first").mirror,
        Mirror::Horizontal
    );
}

#[test]
fn rotate_group_rolls_back_on_collision() {
    let mut document = Document::new();
    let first = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(&RESISTOR, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    document.add(&RESISTOR, GridPos::new(1, 3), Rotation::Deg0, Mirror::None);
    assert!(!document.rotate_components(LIBRARY, &[first, second]));
    assert_eq!(
        document.component(first).expect("first").rotation,
        Rotation::Deg0
    );
    assert_eq!(
        document.component(second).expect("second").rotation,
        Rotation::Deg0
    );
}

#[test]
fn add_generates_unique_labels() {
    let mut document = Document::new();
    let first = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(&RESISTOR, GridPos::new(5, 0), Rotation::Deg0, Mirror::None);
    assert_eq!(document.component(first).expect("first").label, "R1");
    assert_eq!(document.component(second).expect("second").label, "R2");
}

#[test]
fn add_with_value_preserves_value() {
    let mut document = Document::new();
    let id = document.add_with_value(
        &RESISTOR,
        GridPos::new(0, 0),
        Rotation::Deg0,
        Mirror::None,
        "10k",
    );
    let instance = document.component(id).expect("instance");
    assert_eq!(instance.value, "10k");
    assert_eq!(instance.label, "R1");
}

#[test]
fn hit_test_finds_topmost_component() {
    let mut document = Document::new();
    let id = document.add(&RESISTOR, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    assert_eq!(document.hit_test(LIBRARY, GridPos::new(2, 1)), Some(id));
    assert_eq!(document.hit_test(LIBRARY, GridPos::new(10, 10)), None);
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

#[test]
#[ignore = "performance benchmark"]
fn benchmark_large_document() {
    let library = library::builtin();
    let started = std::time::Instant::now();
    let document = large_document();
    run_large_queries(&document, library);
    println!("large document benchmark: {:?}", started.elapsed());
}

fn large_document() -> Document {
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut source = Document::new();
    for row in 0..20 {
        for column in 0..25 {
            let position = GridPos::new(column * 4, row * 3);
            source.add(resistor, position, Rotation::Deg0, Mirror::None);
        }
    }
    let wires = (0..2000).map(benchmark_wire).collect();
    Document::restore(source.components().to_vec(), wires)
}

fn benchmark_wire(index: usize) -> Wire {
    let start = GridPos::new(((index % 10) * 4) as i32, ((index / 10) * 2) as i32);
    let end = start.offset(GridPos::new(2, 0));
    test_wire(index as u32, start, end, NetClass::Digital)
}

fn run_large_queries(document: &Document, library: &[ComponentDefinition]) {
    for index in 0..1000 {
        let point = [(index % 40) as f32, (index / 40) as f32 * 2.0];
        let _ = document.wire_at(point, 0.35);
    }
    let rect = super::GridRect::new(GridPos::new(0, 0), GridSize::new(50, 30));
    let _ = document.components_in_rect(library, rect);
}

#[test]
fn wire_hit_index_follows_updated_points() {
    let library = library::builtin();
    let (mut document, wire) = segment_document();
    assert_eq!(document.wire_at([5.0, 0.2], 0.3), Some(wire));
    let moved = vec![
        GridPos::new(4, 1),
        GridPos::new(4, 2),
        GridPos::new(6, 2),
        GridPos::new(6, 1),
    ];
    assert!(document.update_wire_points(library, wire, moved));
    assert_eq!(document.wire_at([5.0, 0.2], 0.3), None);
    assert_eq!(document.wire_at([5.0, 2.2], 0.3), Some(wire));
}

#[test]
fn wire_hit_index_rebuilds_on_restore() {
    let cable = test_wire(3, GridPos::new(0, 0), GridPos::new(4, 0), NetClass::Digital);
    let document = Document::restore(Vec::new(), vec![cable]);
    assert_eq!(document.wire_at([2.0, 0.2], 0.3), Some(WireId(3)));
    assert_eq!(
        document.wire_segment_at(WireId(3), [2.0, 0.2], 0.3),
        Some(0)
    );
}

#[test]
fn wire_hit_index_drops_removed_wires() {
    let cable = test_wire(4, GridPos::new(0, 0), GridPos::new(4, 0), NetClass::Digital);
    let mut document = Document::restore(Vec::new(), vec![cable]);
    document.remove_wire(WireId(4));
    assert_eq!(document.wire_at([2.0, 0.2], 0.3), None);
}

fn segment_document() -> (Document, super::WireId) {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let points = vec![
        GridPos::new(4, 1),
        GridPos::new(4, 0),
        GridPos::new(6, 0),
        GridPos::new(6, 1),
    ];
    let wire = document
        .add_wire(library, points, NetClass::Digital)
        .expect("wire");
    (document, wire)
}

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
