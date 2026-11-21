use super::*;

impl GuiApp {
    pub(crate) fn canvas(&mut self, ui: &mut egui::Ui) -> canvas::CanvasState {
        let state = self.scene_state(ui.ctx().input(|input| input.time));
        let refs = SceneRefs {
            document: &self.document,
            library: self.library,
            selection: &self.selection,
            wire_preview: &self.wire_points,
        };
        let scene = build_scene(refs, state);
        let grid = self.grid_options();
        let camera = &mut self.camera;
        let initialized = &mut self.initialized;
        show_canvas(ui, camera, grid, initialized, scene)
    }

    fn grid_options(&self) -> canvas::GridOptions {
        canvas::GridOptions {
            visible: self.grid_visible,
            step: self.grid_step.value(),
            style: self.grid_style,
            axes: self.grid_axes,
            rulers: self.grid_rulers,
        }
    }

    pub(crate) fn scene_state(&self, time: f64) -> SceneState {
        SceneState {
            hover_wire: self
                .pointer_world
                .and_then(|point| self.document.wire_at([point.x, point.y], 0.35)),
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
            wire_class: self.wire_class,
            selection_box: self.selection_box(),
            snap: self.pointer_world.map(|world| self.snap_world(world)),
            minimap: self.minimap_open,
            flow_phase: flow_phase(self.flow_animation, time),
        }
    }

    pub(crate) fn handle_canvas(&mut self, state: canvas::CanvasState) {
        self.canvas_focused = state.focused;
        if self.placing.is_some() {
            self.handle_placement(state);
            return;
        }
        if self.tool == Tool::Wire {
            self.handle_wire(state);
            return;
        }
        if self.tool == Tool::Zoom {
            self.handle_zoom_box(&state);
            return;
        }
        self.handle_select_canvas(&state);
    }

    pub(crate) fn handle_select_canvas(&mut self, state: &canvas::CanvasState) {
        self.begin_drag(state);
        self.drag_move(state);
        self.end_drag(state);
        if state.secondary_clicked {
            self.open_context_menu(state);
            return;
        }
        let Some(world) = state.click_world.filter(|_| state.clicked) else {
            return;
        };
        self.select_at(world, state.multi_select);
    }
}

mod drag;
mod keys;
mod placement;
mod selection;
mod transform;
mod wire;
mod zoom;
