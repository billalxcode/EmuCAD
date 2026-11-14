use super::GridCamera;
use super::grid::grid_step;
use eframe::egui;

fn rect() -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(800.0, 600.0))
}

#[test]
fn screen_world_round_trip() {
    let camera = GridCamera::default();
    let world = egui::pos2(3.0, -2.0);
    let screen = camera.world_to_screen(rect(), world);
    let result = camera.screen_to_world(rect(), screen);
    assert!((result.x - world.x).abs() < 1e-5);
    assert!((result.y - world.y).abs() < 1e-5);
}

#[test]
fn zoom_keeps_cursor_world_point() {
    let mut camera = GridCamera::default();
    let cursor = egui::pos2(200.0, 180.0);
    let before = camera.screen_to_world(rect(), cursor);
    camera.zoom_at(rect(), cursor, 1.5);
    let after = camera.screen_to_world(rect(), cursor);
    assert!((before.x - after.x).abs() < 1e-5);
    assert!((before.y - after.y).abs() < 1e-5);
}

#[test]
fn fit_world_centers_and_scales() {
    let mut camera = crate::canvas::GridCamera::default();
    let rect = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(200.0, 100.0),
    );
    camera.fit_world(
        rect,
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::pos2(10.0, 10.0),
        0.0,
    );
    assert!((camera.zoom - 0.5).abs() < 1e-6);
    let center = camera.world_to_screen(rect, eframe::egui::pos2(5.0, 5.0));
    assert!((center - rect.center()).length() < 1e-4);
}

#[test]
fn zoom_is_clamped() {
    let mut camera = GridCamera::default();
    camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 100.0);
    assert_eq!(camera.zoom, 4.0);
    camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 0.001);
    assert_eq!(camera.zoom, 0.25);
}

#[test]
fn grid_step_keeps_readable_spacing() {
    assert_eq!(grid_step(20.0), 1);
    assert_eq!(grid_step(8.0), 2);
    assert_eq!(grid_step(5.0), 5);
}
