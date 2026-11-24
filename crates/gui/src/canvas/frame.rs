use super::*;
use crate::design;

/// Draws the canvas and applies pan and zoom input.
pub fn show(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid: GridOptions,
    initialized: &mut bool,
    draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
) -> CanvasState {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = response.rect;
    let pointer = handle_viewport_input(ui, &response, camera, rect, initialized);
    let frame = CanvasFrame::new(&response, &painter, rect, camera, grid, pointer);
    frame.paint(draw_scene);
    canvas_state(ui, &response, rect, camera, pointer)
}

fn handle_viewport_input(
    ui: &egui::Ui,
    response: &egui::Response,
    camera: &mut GridCamera,
    rect: egui::Rect,
    initialized: &mut bool,
) -> Option<egui::Pos2> {
    initialize_camera(rect, camera, initialized);
    handle_pan(response, ui, camera);
    handle_scroll(response, ui, camera, rect);
    handle_zoom_keys(response, ui, camera, rect);
    focus_canvas(response);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    pointer_world(response, rect, camera)
}

struct CanvasFrame<'a> {
    response: &'a egui::Response,
    painter: &'a egui::Painter,
    rect: egui::Rect,
    camera: &'a GridCamera,
    grid: GridOptions,
    pointer: Option<egui::Pos2>,
}

impl<'a> CanvasFrame<'a> {
    fn new(
        response: &'a egui::Response,
        painter: &'a egui::Painter,
        rect: egui::Rect,
        camera: &'a GridCamera,
        grid: GridOptions,
        pointer: Option<egui::Pos2>,
    ) -> Self {
        Self {
            response,
            painter,
            rect,
            camera,
            grid,
            pointer,
        }
    }

    fn paint(
        self,
        draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
    ) {
        let painter = self.painter;
        let palette = design::palette_for(painter.ctx());
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, palette.canvas);
        draw_grid(painter, self.rect, self.camera, self.grid, &palette);
        draw_scene(painter, self.rect, self.camera, self.pointer);
        paint_focus_ring(self.response, painter, self.rect, &palette);
    }
}

fn focus_canvas(response: &egui::Response) {
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
}

fn paint_focus_ring(
    response: &egui::Response,
    painter: &egui::Painter,
    rect: egui::Rect,
    palette: &design::Palette,
) {
    if !response.has_focus() {
        return;
    }
    painter.rect_stroke(
        rect.shrink(1.0),
        0.0,
        egui::Stroke::new(2.0, palette.focus),
        egui::StrokeKind::Inside,
    );
}
