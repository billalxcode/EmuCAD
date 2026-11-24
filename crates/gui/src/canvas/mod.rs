//! Canvas camera, grid, pan, and zoom.

use eframe::egui;
use serde::{Deserialize, Serialize};

/// Visual style of the grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

/// Logical grid multiplier used for drawing and snapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GridStep(u32);

impl GridStep {
    pub(crate) const ALL: [Self; 4] = [Self(1), Self(2), Self(5), Self(10)];

    /// Returns the integer multiplier.
    #[must_use]
    pub(crate) const fn value(self) -> u32 {
        self.0
    }

    /// Returns a short label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self.0 {
            2 => "2",
            5 => "5",
            10 => "10",
            _ => "1",
        }
    }
}

impl Default for GridStep {
    fn default() -> Self {
        Self(1)
    }
}

const MILS_PER_UNIT: f64 = 50.0;
const MM_PER_MIL: f64 = 0.0254;
const MM_PER_UNIT: f64 = MILS_PER_UNIT * MM_PER_MIL;

/// Display units for grid coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Units {
    #[default]
    Mils,
    Millimeters,
    Logical,
}

impl Units {
    pub(crate) const ALL: [Self; 3] = [Self::Mils, Self::Millimeters, Self::Logical];

    /// Returns the status label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Mils => "mils",
            Self::Millimeters => "mm",
            Self::Logical => "u",
        }
    }

    /// Formats one grid coordinate.
    #[must_use]
    pub(crate) fn format_coord(self, value: i32) -> String {
        match self {
            Self::Mils => format!("{}", value as i64 * MILS_PER_UNIT as i64),
            Self::Millimeters => format!("{:.2}", value as f64 * MM_PER_UNIT),
            Self::Logical => format!("{value}"),
        }
    }

    /// Formats a distance in logical grid units.
    #[must_use]
    pub(crate) fn format_distance(self, value: f64) -> String {
        match self {
            Self::Mils => format!("{:.0}", value * MILS_PER_UNIT),
            Self::Millimeters => format!("{:.2}", value * MM_PER_UNIT),
            Self::Logical => format!("{value:.1}"),
        }
    }
}

/// When grid snapping applies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SnapMode {
    #[default]
    Always,
    WithGrid,
    Never,
}

impl SnapMode {
    pub(crate) const ALL: [Self; 3] = [Self::Always, Self::WithGrid, Self::Never];

    /// Returns a short label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Always => "Always",
            Self::WithGrid => "With grid",
            Self::Never => "Never",
        }
    }
}

/// Crosshair shape drawn at the snapped cursor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CrosshairMode {
    #[default]
    Small,
    Full,
    Diagonal,
}

impl CrosshairMode {
    pub(crate) const ALL: [Self; 3] = [Self::Small, Self::Full, Self::Diagonal];

    /// Returns a short label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Full => "Full",
            Self::Diagonal => "Diagonal",
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
