use super::*;

impl GuiApp {
    pub(crate) fn handle_placement(&mut self, state: canvas::CanvasState) {
        if state.secondary_clicked {
            self.placing = None;
            self.drag_place = false;
            return;
        }
        if !state.clicked {
            return;
        }
        let Some(world) = state.click_world else {
            return;
        };
        self.place_at(world);
    }

    pub(crate) fn place_at(&mut self, world: egui::Pos2) {
        let Some(definition) = self.placing else {
            return;
        };
        let Some(position) = self.placement_target(definition, world) else {
            return;
        };
        self.commit_placement(definition, position);
    }

    pub(crate) fn placement_target(
        &self,
        definition: &ComponentDefinition,
        world: egui::Pos2,
    ) -> Option<GridPos> {
        let position =
            scene::placement_position(definition, world_to_grid(world), self.placement_rotation);
        self.document
            .can_place(
                self.library,
                definition,
                position,
                self.placement_rotation,
                self.placement_mirror,
            )
            .then_some(position)
    }

    pub(crate) fn commit_placement(
        &mut self,
        definition: &'static ComponentDefinition,
        position: GridPos,
    ) {
        self.push_undo();
        let id = self.document.add(
            definition,
            position,
            self.placement_rotation,
            self.placement_mirror,
        );
        self.selection.select_component(id);
        self.placing = None;
        self.drag_place = false;
        self.mark_dirty();
        self.remember_component(definition);
    }

    pub(crate) fn finish_drag_place(&mut self, ctx: &egui::Context) {
        if !self.drag_place || !ctx.input(|input| input.pointer.any_released()) {
            return;
        }
        self.drag_place = false;
        let Some(world) = self.pointer_world else {
            return;
        };
        self.place_at(world);
    }

    pub(crate) fn remember_component(&mut self, definition: &'static ComponentDefinition) {
        self.recent_components
            .retain(|item| item.id != definition.id);
        self.recent_components.insert(0, definition);
        self.recent_components.truncate(5);
    }
}
