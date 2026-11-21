use super::*;

impl GuiApp {
    pub(crate) fn select_at(&mut self, world: egui::Pos2, additive: bool) {
        let point = self.snap_world(world).point;
        if self.select_pin_at(point, additive) {
            return;
        }
        if self.select_component_at(point, additive) {
            return;
        }
        if self.select_wire_at(world, additive) {
            return;
        }
        if !additive {
            self.selection.clear();
        }
    }

    pub(crate) fn select_pin_at(&mut self, point: GridPos, additive: bool) -> bool {
        let Some(id) = self.document.pin_wire_at(self.library, point) else {
            return false;
        };
        self.select_wire_net(id, additive);
        true
    }

    pub(crate) fn select_component_at(&mut self, point: GridPos, additive: bool) -> bool {
        let Some(id) = self.document.hit_test(self.library, point) else {
            return false;
        };
        if additive {
            self.selection.toggle_component(id);
        } else {
            self.selection.select_component(id);
        }
        true
    }

    pub(crate) fn select_wire_at(&mut self, world: egui::Pos2, additive: bool) -> bool {
        let Some(id) = self.document.wire_at([world.x, world.y], 0.35) else {
            return false;
        };
        self.select_wire_net(id, additive);
        true
    }

    pub(crate) fn select_wire_net(&mut self, id: WireId, additive: bool) {
        if additive {
            self.selection.toggle_wire(id);
            return;
        }
        let net = self.document.net_wires(id);
        self.selection.apply([], net, false);
    }

    pub(crate) fn open_context_menu(&mut self, state: &canvas::CanvasState) {
        let Some(click) = state.context_click else {
            return;
        };
        self.select_context_target(click.world);
        self.context_menu = Some(click.screen);
    }

    pub(crate) fn select_context_target(&mut self, world: egui::Pos2) {
        let point = self.snap_world(world).point;
        if self.context_pin(point) {
            return;
        }
        if self.context_component(point) {
            return;
        }
        if self.context_wire(world) {
            return;
        }
        self.selection.clear();
    }

    pub(crate) fn context_pin(&mut self, point: GridPos) -> bool {
        let Some(id) = self.document.pin_wire_at(self.library, point) else {
            return false;
        };
        if !self.selection.contains_wire(id) {
            self.select_wire_net(id, false);
        }
        true
    }

    pub(crate) fn context_component(&mut self, point: GridPos) -> bool {
        let Some(id) = self.document.hit_test(self.library, point) else {
            return false;
        };
        if !self.selection.contains_component(id) {
            self.selection.select_component(id);
        }
        true
    }

    pub(crate) fn context_wire(&mut self, world: egui::Pos2) -> bool {
        let Some(id) = self.document.wire_at([world.x, world.y], 0.35) else {
            return false;
        };
        if !self.selection.contains_wire(id) {
            self.select_wire_net(id, false);
        }
        true
    }
}
