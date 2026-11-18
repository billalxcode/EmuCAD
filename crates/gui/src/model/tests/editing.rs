use super::*;

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
