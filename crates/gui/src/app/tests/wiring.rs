use super::*;

#[test]
fn wire_tool_starts_at_pin() {
    let mut app = wire_app(Vec::new());
    app.handle_canvas(click_state(5.0, 1.0));
    assert_eq!(app.wire_points, vec![GridPos::new(4, 1)]);
}

#[test]
fn wire_tool_extends_to_valid_point() {
    let mut app = wire_app(vec![GridPos::new(4, 1)]);
    app.handle_canvas(click_state(8.0, 1.0));
    assert_eq!(
        app.wire_points,
        vec![GridPos::new(4, 1), GridPos::new(8, 1)]
    );
}

#[test]
fn wire_tool_finishes_at_second_pin() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        tool: crate::app::Tool::Wire,
        wire_points: vec![GridPos::new(4, 1)],
        ..GuiApp::default()
    };
    app.handle_canvas(click_state(6.0, 1.0));
    assert_eq!(app.document.wires().len(), 1);
    assert!(app.wire_points.is_empty());
}

#[test]
fn wire_class_is_applied_to_new_wire() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        tool: crate::app::Tool::Wire,
        wire_class: NetClass::Power,
        wire_points: vec![GridPos::new(4, 1)],
        ..GuiApp::default()
    };
    app.handle_canvas(click_state(6.0, 1.0));
    assert_eq!(app.document.wires()[0].class, NetClass::Power);
}

#[test]
fn selected_wire_class_can_be_changed() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_class(NetClass::Analog);
    assert_eq!(app.document.wires()[0].class, NetClass::Analog);
    app.undo();
    assert_eq!(app.document.wires()[0].class, NetClass::Digital);
}

#[test]
fn selected_wire_color_can_be_overridden() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_color_override(Some([1, 2, 3]));
    assert_eq!(app.document.wires()[0].color_override, Some([1, 2, 3]));
    app.undo();
    assert_eq!(app.document.wires()[0].color_override, None);
}

#[test]
fn selected_wire_label_can_be_set() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_label(Some("LED_A".to_string()));
    assert_eq!(app.document.wires()[0].label.as_deref(), Some("LED_A"));
    app.undo();
    assert_eq!(app.document.wires()[0].label, None);
}
