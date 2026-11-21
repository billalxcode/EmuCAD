use super::*;

impl GuiApp {
    pub(crate) fn begin_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_started || state.space_down {
            return;
        }
        if self.begin_segment_drag(state) {
            return;
        }
        let Some(world) = state.pointer_world else {
            return;
        };
        let point = self.snap_world(world).point;
        let Some(id) = self.document.hit_test(self.library, point) else {
            self.begin_box(point, state.multi_select);
            return;
        };
        self.ensure_selected(id, state.multi_select);
        let offsets = self.drag_offsets(point);
        self.dragging = Some(DragState { offsets });
        self.drag_history_pending = true;
    }

    pub(crate) fn begin_box(&mut self, point: GridPos, additive: bool) {
        self.box_start = Some(point);
        self.box_current = Some(point);
        self.box_additive = additive;
    }

    pub(crate) fn ensure_selected(&mut self, id: ComponentId, additive: bool) {
        if self.selection.contains_component(id) {
            return;
        }
        if additive {
            self.selection.add_component(id);
        } else {
            self.selection.select_component(id);
        }
    }

    pub(crate) fn drag_offsets(&self, point: GridPos) -> Vec<(ComponentId, GridPos)> {
        self.selection
            .components()
            .iter()
            .filter_map(|id| {
                let component = self.document.component(*id)?;
                Some((*id, component.position.subtract(point)))
            })
            .collect()
    }

    pub(crate) fn begin_segment_drag(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(wire_id) = self.selection.single_wire() else {
            return false;
        };
        let Some(world) = state.pointer_world else {
            return false;
        };
        let Some(index) = self.segment_at(wire_id, world) else {
            return false;
        };
        self.dragging_segment = Some(SegmentDrag {
            wire: wire_id,
            index,
        });
        self.drag_history_pending = true;
        true
    }

    pub(crate) fn segment_at(&self, wire_id: WireId, world: egui::Pos2) -> Option<usize> {
        self.document
            .wire_segment_at(wire_id, [world.x, world.y], 0.35)
    }

    pub(crate) fn drag_move(&mut self, state: &canvas::CanvasState) {
        if !state.dragged {
            return;
        }
        if self.drag_segment(state) {
            return;
        }
        if self.update_box(state) {
            return;
        }
        self.move_dragged(state);
    }

    pub(crate) fn move_dragged(&mut self, state: &canvas::CanvasState) {
        let Some(moves) = self.drag_moves(state) else {
            return;
        };
        if !self.any_moved(&moves) {
            return;
        }
        let before = self.document.clone();
        if self.document.move_components(self.library, &moves) {
            self.record_drag(&before);
        }
    }

    pub(crate) fn drag_moves(
        &self,
        state: &canvas::CanvasState,
    ) -> Option<Vec<(ComponentId, GridPos)>> {
        let drag = self.dragging.as_ref()?;
        let world = state.pointer_world?;
        let point = self.snap_world(world).point;
        Some(
            drag.offsets
                .iter()
                .map(|(id, offset)| (*id, point.offset(*offset)))
                .collect(),
        )
    }

    pub(crate) fn any_moved(&self, moves: &[(ComponentId, GridPos)]) -> bool {
        moves.iter().any(|(id, position)| {
            self.document
                .component(*id)
                .is_some_and(|component| component.position != *position)
        })
    }

    pub(crate) fn update_box(&mut self, state: &canvas::CanvasState) -> bool {
        if self.box_start.is_none() {
            return false;
        }
        if let Some(world) = state.pointer_world {
            self.box_current = Some(self.snap_world(world).point);
        }
        true
    }

    pub(crate) fn record_drag(&mut self, before: &Document) {
        if !self.drag_history_pending {
            return;
        }
        self.history.record(before);
        self.drag_history_pending = false;
        self.mark_dirty();
    }

    pub(crate) fn drag_segment(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(drag) = self.dragging_segment else {
            return false;
        };
        let Some(world) = state.pointer_world else {
            return true;
        };
        self.update_segment(drag, world);
        true
    }

    pub(crate) fn update_segment(&mut self, drag: SegmentDrag, world: egui::Pos2) {
        let Some(mut points) = self.segment_points(drag) else {
            return;
        };
        if !move_segment(&mut points, drag.index, self.snap_world(world).point) {
            return;
        }
        let before = self.document.clone();
        if self
            .document
            .update_wire_points(self.library, drag.wire, points)
        {
            self.record_drag(&before);
        }
    }

    pub(crate) fn segment_points(&self, drag: SegmentDrag) -> Option<Vec<GridPos>> {
        self.document
            .wires()
            .iter()
            .find(|wire| wire.id == drag.wire)
            .map(|wire| wire.points.clone())
    }

    pub(crate) fn end_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_stopped {
            return;
        }
        if let (Some(start), Some(end)) = (self.box_start, self.box_current) {
            self.apply_box(start, end, self.box_additive);
        }
        self.box_start = None;
        self.box_current = None;
        self.box_additive = false;
        self.dragging = None;
        self.dragging_segment = None;
        self.drag_history_pending = false;
    }

    pub(crate) fn apply_box(&mut self, start: GridPos, end: GridPos, additive: bool) {
        let rect = GridRect::from_corners(start, end);
        let components = self.document.components_in_rect(self.library, rect);
        let wires = self.document.wires_in_rect(rect);
        self.selection.apply(components, wires, additive);
    }

    pub(crate) fn selection_box(&self) -> Option<(GridPos, GridPos)> {
        Some((self.box_start?, self.box_current?))
    }
}
