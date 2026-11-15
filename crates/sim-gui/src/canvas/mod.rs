//! Canvas camera, grid, pan, and zoom.

use eframe::egui;

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
