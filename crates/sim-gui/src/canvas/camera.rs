use super::*;

/// Screen-space state of the grid camera.
#[derive(Clone, Debug)]
pub struct GridCamera {
    /// Screen offset of the world origin relative to the canvas top-left.
    pub pan: egui::Vec2,
    /// Zoom multiplier applied to the base grid cell size.
    pub zoom: f32,
    /// Base size of one grid cell in pixels at 100 percent zoom.
    pub cell_px: f32,
}

impl Default for GridCamera {
    fn default() -> Self {
        Self {
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            cell_px: 20.0,
        }
    }
}

impl GridCamera {
    /// Returns pixels per world grid unit.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.cell_px * self.zoom
    }

    /// Converts a world grid position to screen space.
    #[must_use]
    pub fn world_to_screen(&self, rect: egui::Rect, world: egui::Pos2) -> egui::Pos2 {
        rect.min + self.pan + world.to_vec2() * self.scale()
    }

    /// Converts a screen position to world grid space.
    #[must_use]
    pub fn screen_to_world(&self, rect: egui::Rect, screen: egui::Pos2) -> egui::Pos2 {
        ((screen - rect.min - self.pan) / self.scale()).to_pos2()
    }

    /// Zooms around a screen position while keeping that world point fixed.
    pub fn zoom_at(&mut self, rect: egui::Rect, cursor: egui::Pos2, factor: f32) {
        let world = self.screen_to_world(rect, cursor);
        self.zoom = (self.zoom * factor).clamp(0.25, 4.0);
        let screen = self.world_to_screen(rect, world);
        self.pan += cursor - screen;
    }

    /// Fits a world-space box into the canvas with padding.
    pub fn fit_world(&mut self, rect: egui::Rect, min: egui::Pos2, max: egui::Pos2, padding: f32) {
        let size = (max - min).max(egui::vec2(1.0, 1.0));
        let available =
            (rect.size() - egui::vec2(padding * 2.0, padding * 2.0)).max(egui::vec2(1.0, 1.0));
        let zoom_x = available.x / (size.x * self.cell_px);
        let zoom_y = available.y / (size.y * self.cell_px);
        self.zoom = zoom_x.min(zoom_y).clamp(0.25, 4.0);
        let center = ((min.to_vec2() + max.to_vec2()) * 0.5).to_pos2();
        self.pan = rect.size() * 0.5 - center.to_vec2() * self.scale();
    }

    pub(super) fn visible_range(&self, rect: egui::Rect, step: i32) -> GridRange {
        let min = self.screen_to_world(rect, rect.min);
        let max = self.screen_to_world(rect, rect.max);
        let step = step as f32;
        GridRange {
            min_x: (min.x / step).floor() as i32,
            max_x: (max.x / step).ceil() as i32,
            min_y: (min.y / step).floor() as i32,
            max_y: (max.y / step).ceil() as i32,
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
    camera.pan = rect.size() * 0.5;
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
    if dragging {
        camera.pan += response.drag_delta();
    }
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
    let Some(cursor) = response.hover_pos() else {
        return;
    };
    if scroll != 0.0 {
        camera.zoom_at(rect, cursor, (scroll * 0.002).exp());
    }
}
