use super::*;

/// Draws the canvas and applies pan and zoom input.
pub fn show(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid_visible: bool,
    initialized: &mut bool,
    draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
) -> CanvasState {
    let (response, painter) =
        ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
    let rect = response.rect;
    initialize_camera(rect, camera, initialized);
    handle_pan(&response, ui, camera);
    handle_zoom(&response, ui, camera, rect);
    let pointer = pointer_world(&response, rect, camera);
    focus_canvas(&response);
    let frame = CanvasFrame::new(&response, &painter, rect, camera, grid_visible, pointer);
    frame.paint(draw_scene);
    canvas_state(ui, &response, rect, camera, pointer)
}

struct CanvasFrame<'a> {
    response: &'a egui::Response,
    painter: &'a egui::Painter,
    rect: egui::Rect,
    camera: &'a GridCamera,
    grid_visible: bool,
    pointer: Option<egui::Pos2>,
}

impl<'a> CanvasFrame<'a> {
    fn new(
        response: &'a egui::Response,
        painter: &'a egui::Painter,
        rect: egui::Rect,
        camera: &'a GridCamera,
        grid_visible: bool,
        pointer: Option<egui::Pos2>,
    ) -> Self {
        Self {
            response,
            painter,
            rect,
            camera,
            grid_visible,
            pointer,
        }
    }

    fn paint(
        self,
        draw_scene: impl FnOnce(&egui::Painter, egui::Rect, &GridCamera, Option<egui::Pos2>),
    ) {
        let painter = self.painter;
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, BACKGROUND);
        if self.grid_visible {
            draw_grid(painter, self.rect, self.camera);
        }
        draw_scene(painter, self.rect, self.camera, self.pointer);
        paint_focus_ring(self.response, painter, self.rect);
    }
}

fn focus_canvas(response: &egui::Response) {
    if response.clicked() || response.drag_started() {
        response.request_focus();
    }
}

fn paint_focus_ring(response: &egui::Response, painter: &egui::Painter, rect: egui::Rect) {
    if !response.has_focus() {
        return;
    }
    painter.rect_stroke(
        rect.shrink(1.0),
        0.0,
        egui::Stroke::new(2.0, FOCUS_COLOR),
        egui::StrokeKind::Inside,
    );
}
