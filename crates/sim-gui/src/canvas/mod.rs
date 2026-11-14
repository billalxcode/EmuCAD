//! Canvas camera, grid, pan, and zoom.

use eframe::egui;

const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(18, 20, 24);
const MINOR_DOT: egui::Color32 = egui::Color32::from_rgb(55, 60, 70);
const MAJOR_LINE: egui::Color32 = egui::Color32::from_rgb(40, 45, 54);
const AXIS_LINE: egui::Color32 = egui::Color32::from_rgb(90, 100, 118);
const FOCUS_COLOR: egui::Color32 = egui::Color32::from_rgb(80, 215, 225);
const RULER_COLOR: egui::Color32 = egui::Color32::from_rgb(145, 155, 170);

/// Screen-space state of the grid camera.
mod camera;
mod frame;
mod grid;
mod state;
#[cfg(test)]
mod tests;

pub use camera::GridCamera;
use camera::{GridRange, handle_pan, handle_zoom, initialize_camera, pointer_world};
pub use frame::show;
use grid::draw_grid;
pub use state::CanvasState;
#[cfg(test)]
pub(crate) use state::ContextClick;
use state::canvas_state;
