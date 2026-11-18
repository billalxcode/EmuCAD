use super::*;

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
