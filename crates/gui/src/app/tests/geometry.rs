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
