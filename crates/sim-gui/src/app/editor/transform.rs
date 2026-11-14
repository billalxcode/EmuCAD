use super::*;

impl GuiApp {
    pub(crate) fn nudge_selection(&mut self, dx: i32, dy: i32) {
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let moves = self.nudge_moves(&ids, dx, dy);
        let mut candidate = self.document.clone();
        if !candidate.move_components(self.library, &moves) {
            return;
        }
        self.push_undo();
        self.document = candidate;
        self.mark_dirty();
        self.message = "Nudge".to_string();
    }

    pub(crate) fn nudge_moves(
        &self,
        ids: &[ComponentId],
        dx: i32,
        dy: i32,
    ) -> Vec<(ComponentId, GridPos)> {
        let delta = GridPos::new(dx, dy);
        ids.iter()
            .filter_map(|id| {
                self.document
                    .component(*id)
                    .map(|component| (*id, component.position.offset(delta)))
            })
            .collect()
    }

    pub(crate) fn rotate(&mut self) {
        if self.placing.is_some() {
            self.placement_rotation = self.placement_rotation.next();
            return;
        }
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.rotate_components(self.library, &ids) {
            self.history.record(&before);
            self.mark_dirty();
        }
    }

    pub(crate) fn mirror(&mut self) {
        if self.placing.is_some() {
            self.placement_mirror = self.placement_mirror.next();
            return;
        }
        let ids: Vec<_> = self.selection.components().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.mirror_components(self.library, &ids) {
            self.history.record(&before);
            self.mark_dirty();
        }
    }

    pub(crate) fn align_selection(&mut self, align: Align) {
        let moves = layout::align_moves(&self.document, self.library, &self.selection, align);
        self.apply_layout_moves(moves);
    }

    pub(crate) fn distribute_selection(&mut self, axis: Distribute) {
        let moves = layout::distribute_moves(&self.document, self.library, &self.selection, axis);
        self.apply_layout_moves(moves);
    }

    pub(crate) fn apply_layout_moves(&mut self, mut moves: Vec<(ComponentId, GridPos)>) {
        moves.retain(|(id, position)| {
            self.document
                .component(*id)
                .is_some_and(|component| component.position != *position)
        });
        if moves.is_empty() {
            return;
        }
        let before = self.document.clone();
        if self.document.move_components(self.library, &moves) {
            self.history.record(&before);
            self.mark_dirty();
            self.message = "Layout applied".to_string();
            return;
        }
        self.message = "Layout blocked by overlap".to_string();
    }
}
