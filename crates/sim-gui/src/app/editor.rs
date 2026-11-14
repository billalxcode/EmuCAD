use super::*;

impl GuiApp {
    pub(super) fn canvas(&mut self, ui: &mut egui::Ui) -> canvas::CanvasState {
        let state = self.scene_state(ui.ctx().input(|input| input.time));
        let refs = SceneRefs {
            document: &self.document,
            library: self.library,
            selection: &self.selection,
            wire_preview: &self.wire_points,
        };
        let scene = build_scene(refs, state);
        let camera = &mut self.camera;
        let grid_visible = self.grid_visible;
        let initialized = &mut self.initialized;
        show_canvas(ui, camera, grid_visible, initialized, scene)
    }

    pub(super) fn scene_state(&self, time: f64) -> SceneState {
        SceneState {
            hover_wire: self
                .pointer_world
                .and_then(|point| self.document.wire_at([point.x, point.y], 0.35)),
            placing: self.placing,
            rotation: self.placement_rotation,
            mirror: self.placement_mirror,
            wire_class: self.wire_class,
            selection_box: self.selection_box(),
            minimap: self.minimap_open,
            flow_phase: flow_phase(self.flow_animation, time),
        }
    }

    pub(super) fn handle_canvas(&mut self, state: canvas::CanvasState) {
        self.canvas_focused = state.focused;
        if self.placing.is_some() {
            self.handle_placement(state);
            return;
        }
        if self.tool == Tool::Wire {
            self.handle_wire(state);
            return;
        }
        self.handle_select_canvas(&state);
    }

    pub(super) fn handle_select_canvas(&mut self, state: &canvas::CanvasState) {
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

    pub(super) fn select_at(&mut self, world: egui::Pos2, additive: bool) {
        let point = world_to_grid(world);
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

    pub(super) fn select_pin_at(&mut self, point: GridPos, additive: bool) -> bool {
        let Some(id) = self.document.pin_wire_at(self.library, point) else {
            return false;
        };
        self.select_wire_net(id, additive);
        true
    }

    pub(super) fn select_component_at(&mut self, point: GridPos, additive: bool) -> bool {
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

    pub(super) fn select_wire_at(&mut self, world: egui::Pos2, additive: bool) -> bool {
        let Some(id) = self.document.wire_at([world.x, world.y], 0.35) else {
            return false;
        };
        self.select_wire_net(id, additive);
        true
    }

    pub(super) fn select_wire_net(&mut self, id: WireId, additive: bool) {
        if additive {
            self.selection.toggle_wire(id);
            return;
        }
        let net = self.document.net_wires(id);
        self.selection.apply([], net, false);
    }

    pub(super) fn open_context_menu(&mut self, state: &canvas::CanvasState) {
        let Some(click) = state.context_click else {
            return;
        };
        self.select_context_target(click.world);
        self.context_menu = Some(click.screen);
    }

    pub(super) fn select_context_target(&mut self, world: egui::Pos2) {
        let point = world_to_grid(world);
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

    pub(super) fn context_pin(&mut self, point: GridPos) -> bool {
        let Some(id) = self.document.pin_wire_at(self.library, point) else {
            return false;
        };
        if !self.selection.contains_wire(id) {
            self.select_wire_net(id, false);
        }
        true
    }

    pub(super) fn context_component(&mut self, point: GridPos) -> bool {
        let Some(id) = self.document.hit_test(self.library, point) else {
            return false;
        };
        if !self.selection.contains_component(id) {
            self.selection.select_component(id);
        }
        true
    }

    pub(super) fn context_wire(&mut self, world: egui::Pos2) -> bool {
        let Some(id) = self.document.wire_at([world.x, world.y], 0.35) else {
            return false;
        };
        if !self.selection.contains_wire(id) {
            self.select_wire_net(id, false);
        }
        true
    }

    pub(super) fn begin_drag(&mut self, state: &canvas::CanvasState) {
        if !state.drag_started || state.space_down {
            return;
        }
        if self.begin_segment_drag(state) {
            return;
        }
        let Some(world) = state.pointer_world else {
            return;
        };
        let point = world_to_grid(world);
        let Some(id) = self.document.hit_test(self.library, point) else {
            self.begin_box(point, state.multi_select);
            return;
        };
        self.ensure_selected(id, state.multi_select);
        let offsets = self.drag_offsets(point);
        self.dragging = Some(DragState { offsets });
        self.drag_history_pending = true;
    }

    pub(super) fn begin_box(&mut self, point: GridPos, additive: bool) {
        self.box_start = Some(point);
        self.box_current = Some(point);
        self.box_additive = additive;
    }

    pub(super) fn ensure_selected(&mut self, id: ComponentId, additive: bool) {
        if self.selection.contains_component(id) {
            return;
        }
        if additive {
            self.selection.add_component(id);
        } else {
            self.selection.select_component(id);
        }
    }

    pub(super) fn drag_offsets(&self, point: GridPos) -> Vec<(ComponentId, GridPos)> {
        self.selection
            .components()
            .iter()
            .filter_map(|id| {
                let component = self.document.component(*id)?;
                Some((*id, component.position.subtract(point)))
            })
            .collect()
    }

    pub(super) fn begin_segment_drag(&mut self, state: &canvas::CanvasState) -> bool {
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

    pub(super) fn segment_at(&self, wire_id: WireId, world: egui::Pos2) -> Option<usize> {
        self.document
            .wire_segment_at(wire_id, [world.x, world.y], 0.35)
    }

    pub(super) fn drag_move(&mut self, state: &canvas::CanvasState) {
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

    pub(super) fn move_dragged(&mut self, state: &canvas::CanvasState) {
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

    pub(super) fn drag_moves(
        &self,
        state: &canvas::CanvasState,
    ) -> Option<Vec<(ComponentId, GridPos)>> {
        let drag = self.dragging.as_ref()?;
        let world = state.pointer_world?;
        let point = world_to_grid(world);
        Some(
            drag.offsets
                .iter()
                .map(|(id, offset)| (*id, point.offset(*offset)))
                .collect(),
        )
    }

    pub(super) fn any_moved(&self, moves: &[(ComponentId, GridPos)]) -> bool {
        moves.iter().any(|(id, position)| {
            self.document
                .component(*id)
                .is_some_and(|component| component.position != *position)
        })
    }

    pub(super) fn update_box(&mut self, state: &canvas::CanvasState) -> bool {
        if self.box_start.is_none() {
            return false;
        }
        if let Some(world) = state.pointer_world {
            self.box_current = Some(world_to_grid(world));
        }
        true
    }

    pub(super) fn record_drag(&mut self, before: &Document) {
        if !self.drag_history_pending {
            return;
        }
        self.history.record(before);
        self.drag_history_pending = false;
        self.mark_dirty();
    }

    pub(super) fn drag_segment(&mut self, state: &canvas::CanvasState) -> bool {
        let Some(drag) = self.dragging_segment else {
            return false;
        };
        let Some(world) = state.pointer_world else {
            return true;
        };
        self.update_segment(drag, world);
        true
    }

    pub(super) fn update_segment(&mut self, drag: SegmentDrag, world: egui::Pos2) {
        let Some(mut points) = self.segment_points(drag) else {
            return;
        };
        if !move_segment(&mut points, drag.index, world_to_grid(world)) {
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

    pub(super) fn segment_points(&self, drag: SegmentDrag) -> Option<Vec<GridPos>> {
        self.document
            .wires()
            .iter()
            .find(|wire| wire.id == drag.wire)
            .map(|wire| wire.points.clone())
    }

    pub(super) fn end_drag(&mut self, state: &canvas::CanvasState) {
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

    pub(super) fn apply_box(&mut self, start: GridPos, end: GridPos, additive: bool) {
        let rect = GridRect::from_corners(start, end);
        let components = self.document.components_in_rect(self.library, rect);
        let wires = self.document.wires_in_rect(rect);
        self.selection.apply(components, wires, additive);
    }

    pub(super) fn selection_box(&self) -> Option<(GridPos, GridPos)> {
        Some((self.box_start?, self.box_current?))
    }

    pub(super) fn handle_placement(&mut self, state: canvas::CanvasState) {
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

    pub(super) fn place_at(&mut self, world: egui::Pos2) {
        let Some(definition) = self.placing else {
            return;
        };
        let Some(position) = self.placement_target(definition, world) else {
            return;
        };
        self.commit_placement(definition, position);
    }

    pub(super) fn placement_target(
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

    pub(super) fn commit_placement(
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

    pub(super) fn finish_drag_place(&mut self, ctx: &egui::Context) {
        if !self.drag_place || !ctx.input(|input| input.pointer.any_released()) {
            return;
        }
        self.drag_place = false;
        let Some(world) = self.pointer_world else {
            return;
        };
        self.place_at(world);
    }

    pub(super) fn remember_component(&mut self, definition: &'static ComponentDefinition) {
        self.recent_components
            .retain(|item| item.id != definition.id);
        self.recent_components.insert(0, definition);
        self.recent_components.truncate(5);
    }

    pub(super) fn handle_wire(&mut self, state: canvas::CanvasState) {
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

    pub(super) fn wire_click(&mut self, point: GridPos) {
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

    pub(super) fn finish_wire(&mut self) {
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

    pub(super) fn handle_keys(&mut self, ui: &egui::Ui) {
        if self.palette.open {
            return;
        }
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        self.handle_file_keys(ui);
        self.handle_history_keys(ui);
        self.handle_clipboard_keys(ui);
        self.handle_edit_keys(ui);
        self.handle_nudge_keys(ui);
    }

    pub(super) fn handle_history_keys(&mut self, ui: &egui::Ui) {
        let undo = ui.input(|input| {
            input.modifiers.command && !input.modifiers.shift && input.key_pressed(egui::Key::Z)
        });
        let redo = ui.input(|input| {
            input.modifiers.command
                && (input.key_pressed(egui::Key::Y)
                    || (input.modifiers.shift && input.key_pressed(egui::Key::Z)))
        });
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    pub(super) fn handle_clipboard_keys(&mut self, ui: &egui::Ui) {
        let copy = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::C));
        let paste = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::V));
        let duplicate =
            ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::D));
        if copy {
            self.copy_selection();
        }
        if paste {
            self.paste_selection();
        }
        if duplicate {
            self.duplicate_selection();
        }
    }

    pub(super) fn handle_file_keys(&mut self, ui: &egui::Ui) {
        let save = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S));
        let open = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O));
        if save {
            self.save_project();
        }
        if open {
            self.open_project();
        }
    }

    pub(super) fn handle_edit_keys(&mut self, ui: &egui::Ui) {
        self.handle_transform_keys(ui);
        self.handle_wire_keys(ui);
        if ui.input(|input| input.key_pressed(egui::Key::Delete)) {
            self.delete_selection();
        }
    }

    pub(super) fn handle_transform_keys(&mut self, ui: &egui::Ui) {
        if ui.input(|input| input.key_pressed(egui::Key::R)) {
            self.rotate();
        }
        if ui.input(|input| input.key_pressed(egui::Key::M)) {
            self.mirror();
        }
    }

    pub(super) fn handle_wire_keys(&mut self, ui: &egui::Ui) {
        if ui.input(|input| input.key_pressed(egui::Key::Enter)) && self.tool == Tool::Wire {
            self.finish_wire();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Backspace)) && self.tool == Tool::Wire {
            self.wire_points.pop();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.placing = None;
            self.drag_place = false;
            self.wire_points.clear();
        }
    }

    pub(super) fn handle_nudge_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused {
            return;
        }
        let (dx, dy) = nudge_delta(ui);
        if dx == 0 && dy == 0 {
            return;
        }
        self.nudge_selection(dx, dy);
    }

    pub(super) fn nudge_selection(&mut self, dx: i32, dy: i32) {
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

    pub(super) fn nudge_moves(
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

    pub(super) fn rotate(&mut self) {
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

    pub(super) fn mirror(&mut self) {
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

    pub(super) fn align_selection(&mut self, align: Align) {
        let moves = layout::align_moves(&self.document, self.library, &self.selection, align);
        self.apply_layout_moves(moves);
    }

    pub(super) fn distribute_selection(&mut self, axis: Distribute) {
        let moves = layout::distribute_moves(&self.document, self.library, &self.selection, axis);
        self.apply_layout_moves(moves);
    }

    pub(super) fn apply_layout_moves(&mut self, mut moves: Vec<(ComponentId, GridPos)>) {
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
