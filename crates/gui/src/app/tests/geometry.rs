use super::*;

fn test_rect(width: f32, height: f32) -> eframe::egui::Rect {
    eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(width, height),
    )
}

#[test]
fn fit_margin_uses_short_viewport_rule() {
    assert_eq!(fit_margin(test_rect(800.0, 600.0)), 1.10);
    assert_eq!(fit_margin(test_rect(800.0, 900.0)), 1.04);
}

#[test]
fn fit_selection_is_tighter_than_all() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(20, 0), Rotation::Deg0, Mirror::None);
    let id = document.components()[0].id;
    let mut app = GuiApp {
        library,
        document,
        canvas_rect: Some(test_rect(800.0, 600.0)),
        ..GuiApp::default()
    };
    app.fit_view(FitTarget::All);
    let all_zoom = app.camera.zoom;
    app.selection.select_component(id);
    app.fit_view(FitTarget::Selection);
    assert!(app.camera.zoom > all_zoom);
}

#[test]
fn grid_step_rounds_to_multiples() {
    let step = GridStep::ALL[1];
    assert_eq!(
        world_to_grid_step(eframe::egui::pos2(1.1, 3.9), step),
        GridPos::new(2, 4)
    );
    assert_eq!(
        world_to_grid_step(eframe::egui::pos2(-1.1, -3.9), step),
        GridPos::new(-2, -4)
    );
}

#[test]
fn snap_mode_controls_grid_snap() {
    let mut app = GuiApp {
        grid_step: GridStep::ALL[1],
        snap_mode: SnapMode::Always,
        ..GuiApp::default()
    };
    let marker = app.snap_world(eframe::egui::pos2(1.1, 3.9));
    assert_eq!(marker.point, GridPos::new(2, 4));
    assert_eq!(marker.kind, crate::scene::SnapKind::Grid);
    app.snap_mode = SnapMode::Never;
    let marker = app.snap_world(eframe::egui::pos2(1.1, 3.9));
    assert_eq!(marker.point, GridPos::new(1, 4));
    assert_eq!(marker.kind, crate::scene::SnapKind::Off);
    app.grid_visible = false;
    app.snap_mode = SnapMode::WithGrid;
    let marker = app.snap_world(eframe::egui::pos2(1.1, 3.9));
    assert_eq!(marker.kind, crate::scene::SnapKind::Off);
}

fn first_pin(document: &Document, library: &[ComponentDefinition]) -> GridPos {
    let instance = &document.components()[0];
    let definition = library
        .iter()
        .find(|definition| definition.id == instance.definition)
        .expect("definition");
    crate::model::world_pin(instance, definition, &definition.pins[0])
}

#[test]
fn snap_world_finds_pin() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let pin = first_pin(&document, library);
    let app = GuiApp {
        library,
        document,
        snap_mode: SnapMode::Never,
        snap_objects: true,
        ..GuiApp::default()
    };
    let marker = app.snap_world(eframe::egui::pos2(pin.x as f32 + 0.1, pin.y as f32));
    assert_eq!(marker.point, pin);
    assert_eq!(marker.kind, crate::scene::SnapKind::Pin);
}

fn add_bent_wire(document: &mut Document, library: &[ComponentDefinition]) {
    document
        .add_wire(
            library,
            vec![
                GridPos::new(4, 1),
                GridPos::new(5, 1),
                GridPos::new(5, -3),
                GridPos::new(10, -3),
            ],
            NetClass::Digital,
        )
        .expect("wire");
}

fn wire_document() -> Document {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(10, -4), Rotation::Deg0, Mirror::None);
    add_bent_wire(&mut document, library);
    document
}

#[test]
fn snap_world_finds_wire() {
    let library = crate::library::builtin();
    let app = GuiApp {
        library,
        document: wire_document(),
        snap_objects: true,
        ..GuiApp::default()
    };
    let marker = app.snap_world(eframe::egui::pos2(5.1, 1.1));
    assert_eq!(marker.point, GridPos::new(5, 1));
    assert_eq!(marker.kind, crate::scene::SnapKind::Wire);
}

fn add_junction_wires(document: &mut Document, library: &[ComponentDefinition]) {
    document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(5, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        )
        .expect("horizontal wire");
    document
        .add_wire(
            library,
            vec![GridPos::new(5, 1), GridPos::new(5, 0)],
            NetClass::Power,
        )
        .expect("vertical wire");
}

fn junction_document() -> Document {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let vcc = crate::library::find("vcc").expect("vcc");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    document.add(vcc, GridPos::new(4, -2), Rotation::Deg0, Mirror::None);
    add_junction_wires(&mut document, library);
    document
}

#[test]
fn snap_world_finds_junction() {
    let app = GuiApp {
        library: crate::library::builtin(),
        document: junction_document(),
        snap_objects: true,
        ..GuiApp::default()
    };
    let marker = app.snap_world(eframe::egui::pos2(5.1, 1.1));
    assert_eq!(marker.point, GridPos::new(5, 1));
    assert_eq!(marker.kind, crate::scene::SnapKind::Junction);
}

#[test]
fn status_uses_snapped_coordinates() {
    let mut app = GuiApp {
        grid_step: GridStep::ALL[1],
        units: Units::Logical,
        pointer_world: Some(eframe::egui::pos2(1.1, 3.9)),
        ..GuiApp::default()
    };
    assert_eq!(app.pointer_text(), "X 2 Y 4");
    assert_eq!(app.delta_text(), "dx 2 dy 4 dist 4.5");
    assert!(app.snap_status().contains("grid"));
    app.snap_mode = SnapMode::Never;
    assert!(app.snap_status().contains("off"));
}

#[test]
fn units_format_coordinates() {
    let point = Some(eframe::egui::pos2(2.0, 4.0));
    let mils = GuiApp {
        units: Units::Mils,
        pointer_world: point,
        ..GuiApp::default()
    };
    assert_eq!(mils.pointer_text(), "X 100 Y 200");
    let millimeters = GuiApp {
        units: Units::Millimeters,
        pointer_world: point,
        ..GuiApp::default()
    };
    assert_eq!(millimeters.pointer_text(), "X 2.54 Y 5.08");
}

#[test]
fn delta_uses_local_origin() {
    let relative = GuiApp {
        units: Units::Logical,
        local_origin: GridPos::new(1, 1),
        pointer_world: Some(eframe::egui::pos2(2.0, 4.0)),
        ..GuiApp::default()
    };
    assert_eq!(relative.delta_text(), "dx 1 dy 3 dist 3.2");
}

#[test]
fn content_bounds_cover_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let bounds = content_bounds(&document, library, None).expect("bounds");
    assert_eq!(bounds, (GridPos::new(0, 0), GridPos::new(10, 2)));
}

#[test]
fn l_route_adds_one_corner() {
    let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), true);
    assert_eq!(route, vec![GridPos::new(4, 0), GridPos::new(4, 2)]);
    let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), false);
    assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(4, 2)]);
}

#[test]
fn l_route_handles_aligned_and_same_points() {
    assert_eq!(
        l_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
        vec![GridPos::new(4, 0)]
    );
    assert!(l_route(GridPos::new(0, 0), GridPos::new(0, 0), true).is_empty());
}

#[test]
fn z_route_adds_two_corners() {
    let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), true);
    assert_eq!(
        route,
        vec![GridPos::new(3, 0), GridPos::new(3, 4), GridPos::new(6, 4)]
    );
    let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), false);
    assert_eq!(
        route,
        vec![GridPos::new(0, 2), GridPos::new(6, 2), GridPos::new(6, 4)]
    );
}

#[test]
fn z_route_simplifies_short_and_aligned() {
    let route = z_route(GridPos::new(0, 0), GridPos::new(1, 2), true);
    assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(1, 2)]);
    assert_eq!(
        z_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
        vec![GridPos::new(4, 0)]
    );
    assert!(z_route(GridPos::new(0, 0), GridPos::new(0, 0), false).is_empty());
}

#[test]
fn cycle_at_x_maps_plot_edges() {
    let plot = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(100.0, 40.0),
    );
    assert_eq!(cycle_at_x(plot, (10, 110), 0.0), 10);
    assert_eq!(cycle_at_x(plot, (10, 110), 100.0), 110);
    assert_eq!(cycle_at_x(plot, (10, 110), 50.0), 60);
}

#[test]
fn move_segment_shifts_both_endpoints() {
    let mut points = vec![
        GridPos::new(0, 0),
        GridPos::new(0, 2),
        GridPos::new(4, 2),
        GridPos::new(4, 4),
    ];
    assert!(move_segment(&mut points, 1, GridPos::new(0, 3)));
    assert_eq!(points[1], GridPos::new(0, 3));
    assert_eq!(points[2], GridPos::new(4, 3));
}
