//! Canvas camera, grid, pan, and zoom.

use eframe::egui;

/// Visual style of the grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GridStyle {
    #[default]
    Dots,
    Lines,
    Cross,
}

impl GridStyle {
    pub(crate) const ALL: [Self; 3] = [Self::Dots, Self::Lines, Self::Cross];

    /// Returns a short label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Dots => "Dots",
            Self::Lines => "Lines",
            Self::Cross => "Cross",
        }
    }
}

/// Grid and overlay options for one canvas frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GridOptions {
    pub(crate) visible: bool,
    pub(crate) step: u32,
    pub(crate) style: GridStyle,
    pub(crate) axes: bool,
    pub(crate) rulers: bool,
    pub(crate) line_width: f32,
    pub(crate) min_spacing: f32,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            visible: true,
            step: 1,
            style: GridStyle::Dots,
            axes: false,
            rulers: true,
            line_width: 1.0,
            min_spacing: 10.0,
        }
    }
}

/// Screen-space state of the grid camera.
mod camera;
mod frame;
mod grid;
mod state;
#[cfg(test)]
mod tests;

pub(crate) use camera::ZOOM_PRESETS;
pub use camera::{DVec2, GridCamera};
use camera::{
    GridRange, handle_pan, handle_scroll, handle_zoom_keys, initialize_camera, pointer_world,
};
#[cfg(test)]
pub(crate) use camera::{MAX_ZOOM, MIN_ZOOM};
pub use frame::show;
use grid::draw_grid;
pub use state::CanvasState;
#[cfg(test)]
pub(crate) use state::ContextClick;
use state::canvas_state;
