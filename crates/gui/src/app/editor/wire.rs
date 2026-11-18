use super::*;

impl GuiApp {
    pub(crate) fn handle_wire(&mut self, state: canvas::CanvasState) {
        if state.secondary_clicked {
            self.wire_points.clear();
            return;
        }
        if state.double_clicked {
            self.finish_wire();
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.wire_click(world_to_grid(world));
    }

    pub(crate) fn wire_click(&mut self, point: GridPos) {
        let point = self.document.snap_pin(self.library, point).unwrap_or(point);
        if self.wire_points.is_empty() {
            if self.document.is_connection_point(self.library, point) {
                self.wire_points.push(point);
            }
            return;
        }
        let Some(&last) = self.wire_points.last() else {
            return;
        };
        if point == last || !self.document.can_add_segment(self.library, last, point) {
            return;
        }
        self.wire_points.push(point);
        if self.document.snap_pin(self.library, point).is_some() {
            self.finish_wire();
        }
    }

    pub(crate) fn finish_wire(&mut self) {
        if self.wire_points.len() < 2 {
            self.wire_points.clear();
            return;
        }
        let points = self.wire_points.clone();
        let before = self.document.clone();
        if self
            .document
            .add_wire(self.library, points, self.wire_class)
            .is_some()
        {
            self.history.record(&before);
            self.wire_points.clear();
            self.mark_dirty();
        }
    }
}
