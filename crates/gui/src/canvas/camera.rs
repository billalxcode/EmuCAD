use super::*;

/// Smallest supported zoom factor.
pub(crate) const MIN_ZOOM: f64 = 0.01;
/// Largest supported zoom factor.
pub(crate) const MAX_ZOOM: f64 = 100.0;

/// Eeschema zoom presets, from 5 percent to 10000 percent.
pub(crate) const ZOOM_PRESETS: &[f64] = &[
    0.05, 0.07, 0.1, 0.15, 0.2, 0.3, 0.5, 0.7, 1.0, 1.5, 2.0, 3.0, 4.5, 6.5, 10.0, 15.0, 20.0,
    30.0, 45.0, 65.0, 100.0,
];

/// Double-precision screen-space vector.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DVec2 {
    pub x: f64,
    pub y: f64,
}

impl DVec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Builds a vector from two coordinates.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Screen-space state of the grid camera.
#[derive(Clone, Debug)]
pub struct GridCamera {
    /// Screen offset of the world origin relative to the canvas top-left.
    pub pan: DVec2,
    /// Zoom multiplier applied to the base grid cell size.
    pub zoom: f64,
    /// Base size of one grid cell in pixels at 100 percent zoom.
    pub cell_px: f64,
}

impl Default for GridCamera {
    fn default() -> Self {
        Self {
            pan: DVec2::ZERO,
            zoom: 1.0,
            cell_px: 20.0,
        }
    }
}

impl GridCamera {
    /// Clamps a zoom factor to the supported range.
    #[must_use]
    pub(crate) fn clamp_zoom(zoom: f64) -> f64 {
        zoom.clamp(MIN_ZOOM, MAX_ZOOM)
    }

    /// Returns pixels per world grid unit.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale_f64() as f32
    }

    fn scale_f64(&self) -> f64 {
        self.cell_px * self.zoom
    }

    /// Converts a world grid position to screen space.
    #[must_use]
    pub fn world_to_screen(&self, rect: egui::Rect, world: egui::Pos2) -> egui::Pos2 {
        let scale = self.scale_f64();
        let x = rect.min.x as f64 + self.pan.x + world.x as f64 * scale;
        let y = rect.min.y as f64 + self.pan.y + world.y as f64 * scale;
        egui::pos2(x as f32, y as f32)
    }

    /// Converts a screen position to world grid space.
    #[must_use]
    pub fn screen_to_world(&self, rect: egui::Rect, screen: egui::Pos2) -> egui::Pos2 {
        let (x, y) = self.screen_to_world_f64(rect, screen);
        egui::pos2(x as f32, y as f32)
    }

    fn screen_to_world_f64(&self, rect: egui::Rect, screen: egui::Pos2) -> (f64, f64) {
        let scale = self.scale_f64();
        (
            (screen.x as f64 - rect.min.x as f64 - self.pan.x) / scale,
            (screen.y as f64 - rect.min.y as f64 - self.pan.y) / scale,
        )
    }

    /// Zooms around a screen position while keeping that world point fixed.
    pub fn zoom_at(&mut self, rect: egui::Rect, cursor: egui::Pos2, factor: f64) {
        let (world_x, world_y) = self.screen_to_world_f64(rect, cursor);
        self.zoom = Self::clamp_zoom(self.zoom * factor);
        let scale = self.scale_f64();
        let x = rect.min.x as f64 + self.pan.x + world_x * scale;
        let y = rect.min.y as f64 + self.pan.y + world_y * scale;
        self.pan.x += cursor.x as f64 - x;
        self.pan.y += cursor.y as f64 - y;
    }

    /// Fits a world-space box into the canvas with padding.
    pub fn fit_world(&mut self, rect: egui::Rect, min: egui::Pos2, max: egui::Pos2, padding: f32) {
        let size = (max - min).max(egui::vec2(1.0, 1.0));
        let available =
            (rect.size() - egui::vec2(padding * 2.0, padding * 2.0)).max(egui::vec2(1.0, 1.0));
        let zoom_x = available.x as f64 / (size.x as f64 * self.cell_px);
        let zoom_y = available.y as f64 / (size.y as f64 * self.cell_px);
        self.zoom = Self::clamp_zoom(zoom_x.min(zoom_y));
        self.center_world(rect, min, max);
    }

    fn center_world(&mut self, rect: egui::Rect, min: egui::Pos2, max: egui::Pos2) {
        let center = ((min.to_vec2() + max.to_vec2()) * 0.5).to_pos2();
        let scale = self.scale_f64();
        self.pan = DVec2::new(
            rect.width() as f64 * 0.5 - center.x as f64 * scale,
            rect.height() as f64 * 0.5 - center.y as f64 * scale,
        );
    }

    pub(super) fn visible_range(&self, rect: egui::Rect, step: i32) -> GridRange {
        let (min_x, min_y) = self.screen_to_world_f64(rect, rect.min);
        let (max_x, max_y) = self.screen_to_world_f64(rect, rect.max);
        let step = step as f64;
        GridRange {
            min_x: (min_x / step).floor() as i32,
            max_x: (max_x / step).ceil() as i32,
            min_y: (min_y / step).floor() as i32,
            max_y: (max_y / step).ceil() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GridRange {
    pub(super) min_x: i32,
    pub(super) max_x: i32,
    pub(super) min_y: i32,
    pub(super) max_y: i32,
}

pub(super) fn initialize_camera(rect: egui::Rect, camera: &mut GridCamera, initialized: &mut bool) {
    if *initialized {
        return;
    }
    camera.pan = DVec2::new(rect.width() as f64 * 0.5, rect.height() as f64 * 0.5);
    *initialized = true;
}

pub(super) fn pointer_world(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<egui::Pos2> {
    response
        .hover_pos()
        .map(|position| camera.screen_to_world(rect, position))
}

pub(super) fn handle_pan(response: &egui::Response, ui: &egui::Ui, camera: &mut GridCamera) {
    let space = ui.input(|input| input.key_down(egui::Key::Space));
    let dragging = response.dragged_by(egui::PointerButton::Middle)
        || (space && response.dragged_by(egui::PointerButton::Primary));
    if !dragging {
        return;
    }
    let delta = response.drag_delta();
    camera.pan.x += delta.x as f64;
    camera.pan.y += delta.y as f64;
}

pub(super) fn handle_zoom(
    response: &egui::Response,
    ui: &egui::Ui,
    camera: &mut GridCamera,
    rect: egui::Rect,
) {
    if !response.hovered() {
        return;
    }
    let scroll = ui.input(|input| input.smooth_scroll_delta.y);
    if scroll == 0.0 {
        return;
    }
    let Some(cursor) = response.hover_pos() else {
        return;
    };
    camera.zoom_at(rect, cursor, (scroll as f64 * 0.002).exp());
}
