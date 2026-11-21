use super::*;

impl GuiApp {
    pub(crate) fn handle_zoom_box(&mut self, state: &canvas::CanvasState) {
        if state.secondary_clicked {
            self.open_context_menu(state);
            return;
        }
        self.begin_zoom_box(state);
        self.update_zoom_box(state);
        self.end_zoom_box(state);
    }

    fn begin_zoom_box(&mut self, state: &canvas::CanvasState) {
        if !state.drag_started || state.space_down {
            return;
        }
        let Some(world) = state.pointer_world else {
            return;
        };
        let point = world_to_grid(world);
        self.box_start = Some(point);
        self.box_current = Some(point);
        self.box_additive = false;
    }

    fn update_zoom_box(&mut self, state: &canvas::CanvasState) {
        if !state.dragged || self.box_start.is_none() {
            return;
        }
        let Some(world) = state.pointer_world else {
            return;
        };
        self.box_current = Some(world_to_grid(world));
    }

    fn end_zoom_box(&mut self, state: &canvas::CanvasState) {
        if !state.drag_stopped {
            return;
        }
        let Some((min, max)) = self.take_zoom_box() else {
            return;
        };
        let Some(rect) = self.canvas_rect else {
            return;
        };
        self.camera
            .fit_world(rect, grid_pos_f(min), grid_pos_f(max), 1.0);
    }

    fn take_zoom_box(&mut self) -> Option<(GridPos, GridPos)> {
        let start = self.box_start.take()?;
        let end = self.box_current.take()?;
        let min = GridPos::new(start.x.min(end.x), start.y.min(end.y));
        let max = GridPos::new(start.x.max(end.x), start.y.max(end.y));
        (min != max).then_some((min, max))
    }
}
