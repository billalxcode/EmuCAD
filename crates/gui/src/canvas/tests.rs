use super::grid::{grid_step, is_major};
use super::{DVec2, GridCamera, GridOptions, GridStyle, MAX_ZOOM, MIN_ZOOM, ZOOM_PRESETS};
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
fn fit_world_margin_reduces_zoom() {
    let mut camera = GridCamera::default();
    let rect = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(200.0, 100.0),
    );
    camera.fit_world(
        rect,
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::pos2(10.0, 10.0),
        1.1,
    );
    assert!(camera.zoom < 0.5);
}

#[test]
fn zoom_is_clamped() {
    let mut camera = GridCamera::default();
    camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 100.0);
    assert_eq!(camera.zoom, MAX_ZOOM);
    camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 0.000001);
    assert_eq!(camera.zoom, MIN_ZOOM);
}

#[test]
fn zoom_presets_are_sorted_and_bounded() {
    assert!(ZOOM_PRESETS.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        ZOOM_PRESETS
            .iter()
            .all(|zoom| (MIN_ZOOM..=MAX_ZOOM).contains(zoom))
    );
    assert_eq!(ZOOM_PRESETS.first(), Some(&0.05));
    assert_eq!(ZOOM_PRESETS.last(), Some(&MAX_ZOOM));
}

#[test]
fn high_zoom_round_trip_keeps_position() {
    let camera = GridCamera {
        pan: DVec2::new(123.456, -78.9),
        zoom: MAX_ZOOM,
        cell_px: 20.0,
        zoom_history: Vec::new(),
    };
    let world = egui::pos2(1234.5, -567.25);
    let screen = camera.world_to_screen(rect(), world);
    let result = camera.screen_to_world(rect(), screen);
    assert!((result.x - world.x).abs() < 1e-2);
    assert!((result.y - world.y).abs() < 1e-2);
}

#[test]
fn fit_world_clamps_to_min_zoom() {
    let mut camera = GridCamera::default();
    camera.fit_world(
        rect(),
        egui::pos2(0.0, 0.0),
        egui::pos2(100_000.0, 100_000.0),
        0.0,
    );
    assert_eq!(camera.zoom, MIN_ZOOM);
}

#[test]
fn scroll_notches_map_to_zoom_steps() {
    assert!((GridCamera::zoom_factor(50.0) - 1.24).abs() < 1e-9);
    assert!((GridCamera::zoom_factor(-50.0) - 1.0 / 1.24).abs() < 1e-9);
    assert_eq!(GridCamera::zoom_factor(0.0), 1.0);
}

#[test]
fn wheel_pan_is_viewport_relative() {
    assert!((GridCamera::pan_delta(50.0, 1000.0) - 120.0).abs() < 1e-9);
    assert_eq!(GridCamera::pan_delta(0.0, 1000.0), 0.0);
}

#[test]
fn zoom_history_restores_previous_view() {
    let mut camera = GridCamera::default();
    camera.zoom_at(rect(), egui::pos2(100.0, 100.0), 2.0);
    let first_zoom = camera.zoom;
    let first_pan = camera.pan;
    camera.zoom_at(rect(), egui::pos2(200.0, 200.0), 2.0);
    assert!(camera.undo_zoom());
    assert_eq!(camera.zoom, first_zoom);
    assert_eq!(camera.pan, first_pan);
    assert!(camera.undo_zoom());
    assert_eq!(camera.zoom, 1.0);
    assert!(!camera.undo_zoom());
}

#[test]
fn zoom_history_keeps_eight_steps() {
    let mut camera = GridCamera::default();
    for _ in 0..12 {
        camera.zoom_at(rect(), rect().center(), 1.1);
    }
    assert_eq!(camera.zoom_history.len(), 8);
}

fn render_grid_style(style: GridStyle) {
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |context| {
        let painter = context.layer_painter(egui::LayerId::background());
        let options = GridOptions {
            style,
            axes: true,
            rulers: false,
            ..Default::default()
        };
        super::grid::draw_grid(
            &painter,
            rect(),
            &GridCamera::default(),
            options,
            &crate::design::DARK,
        );
    });
    output.textures_delta.clear();
}

#[test]
fn grid_styles_render() {
    for style in GridStyle::ALL {
        render_grid_style(style);
    }
}

#[test]
fn grid_step_keeps_readable_spacing() {
    assert_eq!(grid_step(20.0, 1, 10.0), 1);
    assert_eq!(grid_step(8.0, 1, 10.0), 10);
    assert_eq!(grid_step(5.0, 1, 10.0), 10);
    assert_eq!(grid_step(20.0, 2, 10.0), 2);
    assert_eq!(grid_step(20.0, 5, 10.0), 5);
    assert_eq!(grid_step(8.0, 1, 5.0), 1);
}

#[test]
fn coarse_ticks_land_every_ten_steps() {
    assert!(is_major(0));
    assert!(is_major(10));
    assert!(is_major(-20));
    assert!(!is_major(9));
    assert!(!is_major(-1));
}

#[test]
fn visible_range_covers_viewport() {
    let camera = GridCamera {
        pan: DVec2::ZERO,
        zoom: 1.0,
        cell_px: 20.0,
        zoom_history: Vec::new(),
    };
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(200.0, 100.0));
    let range = camera.visible_range(rect, 1);
    assert_eq!(
        (range.min_x, range.max_x, range.min_y, range.max_y),
        (0, 10, 0, 5)
    );
    let range = camera.visible_range(rect, 2);
    assert_eq!(
        (range.min_x, range.max_x, range.min_y, range.max_y),
        (0, 5, 0, 3)
    );
}
