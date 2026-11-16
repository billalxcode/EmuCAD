use super::*;

#[test]
fn additive_click_collects_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.select_at(eframe::egui::pos2(1.0, 1.0), false);
    assert_eq!(app.selection.single_component(), Some(first));
    app.select_at(eframe::egui::pos2(7.0, 1.0), true);
    assert_eq!(app.selection.components().len(), 2);
}

#[test]
fn drag_offsets_keep_relative_positions() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.apply([first, second], [], false);
    let offsets = app.drag_offsets(GridPos::new(1, 0));
    assert_eq!(
        offsets,
        vec![(first, GridPos::new(-1, 0)), (second, GridPos::new(5, 0))]
    );
}

#[test]
fn selection_box_selects_components_and_wires() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let wire = sample_wire(&mut document, library);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.apply_box(GridPos::new(-1, -1), GridPos::new(10, 3), false);
    assert_eq!(app.selection.components().len(), 2);
    assert!(app.selection.contains_wire(wire));
}

#[test]
fn copy_and_paste_selects_new_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.select_component(first);
    app.copy_selection();
    app.paste_selection();
    assert_eq!(app.document.components().len(), 2);
    assert_eq!(app.selection.components().len(), 1);
    assert_ne!(app.selection.single_component(), Some(first));
}

#[test]
fn delete_confirmation_for_wired_or_large_selection() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    assert!(!app.needs_delete_confirmation());
    sample_wire(&mut app.document, app.library);
    assert!(app.needs_delete_confirmation());
    let large: Vec<_> = (0..6u32).map(ComponentId).collect();
    app.selection.apply(large, [], false);
    assert!(app.needs_delete_confirmation());
}

#[test]
fn place_at_commits_definition() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut app = GuiApp {
        library,
        placing: Some(resistor),
        drag_place: true,
        ..GuiApp::default()
    };
    app.place_at(eframe::egui::pos2(0.0, 0.0));
    assert_eq!(app.document.components().len(), 1);
    assert!(app.placing.is_none());
    assert!(!app.drag_place);
}

#[test]
fn nudge_selection_moves_component() {
    let (mut app, id) = nudged_app();
    app.nudge_selection(1, 0);
    assert_component_position(&app, id, GridPos::new(1, 0));
    app.nudge_selection(0, 2);
    assert_component_position(&app, id, GridPos::new(1, 2));
}

#[test]
fn every_tool_has_an_icon() {
    for tool in super::Tool::ALL {
        assert!(!tool.icon().is_empty());
        assert!(!tool.label().is_empty());
    }
}

fn nudged_app() -> (GuiApp, ComponentId) {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut app = GuiApp {
        library,
        ..GuiApp::default()
    };
    let id = app
        .document
        .add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    app.selection.select_component(id);
    (app, id)
}

fn assert_component_position(app: &GuiApp, id: ComponentId, expected: GridPos) {
    assert_eq!(
        app.document.component(id).expect("component").position,
        expected
    );
}

#[test]
fn component_value_change_marks_dirty() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let id = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.select_component(id);
    app.set_component_parameter("resistance", 330.0);
    let component = app.document.component(id).expect("component");
    assert_eq!(component.parameters.get("resistance"), Some(&330.0));
    assert_eq!(component.value, "330 ohm");
    assert!(app.dirty);
    assert_eq!(app.message, "Value updated; rerun DC");
}

#[test]
fn non_primary_parameter_keeps_primary_value() {
    let library = crate::library::builtin();
    let diode = crate::library::find("diode.rectifier.rectifier_diode").expect("diode");
    let mut document = Document::new();
    let id = document.add(diode, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.select_component(id);
    app.set_component_parameter("n", 2.0);
    let component = app.document.component(id).expect("component");
    assert_eq!(component.parameters.get("n"), Some(&2.0));
    assert_eq!(component.value, "1e-12 A");
    assert!(app.dirty);
}
