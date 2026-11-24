use super::*;

impl GuiApp {
    pub(crate) fn handle_keys(&mut self, ui: &egui::Ui) {
        if self.palette.open {
            return;
        }
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        self.handle_file_keys(ui);
        self.handle_history_keys(ui);
        self.handle_clipboard_keys(ui);
        self.handle_tool_keys(ui);
        self.handle_panel_keys(ui);
        self.handle_edit_keys(ui);
        self.handle_cursor_keys(ui);
        self.handle_nudge_keys(ui);
        self.handle_view_keys(ui);
    }

    pub(crate) fn handle_cursor_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused || ui.input(|input| input.modifiers.shift) {
            return;
        }
        let step = self.cursor_step(ui);
        let (dx, dy) = cursor_delta(ui, step);
        if dx == 0 && dy == 0 {
            return;
        }
        let base = self
            .pointer_world
            .map(|world| self.snap_world(world).point)
            .unwrap_or_default();
        self.warp_cursor(ui, base.offset(GridPos::new(dx, dy)));
    }

    fn cursor_step(&self, ui: &egui::Ui) -> i32 {
        let step = self.grid_step.value() as i32;
        if ui.input(|input| input.modifiers.command) {
            step * 10
        } else {
            step
        }
    }

    fn warp_cursor(&self, ui: &egui::Ui, target: GridPos) {
        let Some(rect) = self.canvas_rect else {
            return;
        };
        let world = egui::pos2(target.x as f32, target.y as f32);
        let screen = self.camera.world_to_screen(rect, world);
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::CursorPosition(screen));
    }

    pub(crate) fn handle_view_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused {
            return;
        }
        if self.handle_fast_grid(ui) {
            return;
        }
        self.handle_fit_keys(ui);
    }

    fn handle_fast_grid(&mut self, ui: &egui::Ui) -> bool {
        let (fast1, fast2) = ui.input(|input| {
            let fast1 = input.modifiers.alt && input.key_pressed(egui::Key::Num1);
            let fast2 = input.modifiers.alt && input.key_pressed(egui::Key::Num2);
            (fast1, fast2)
        });
        if fast1 {
            self.grid_step = self.fast_grid_1;
            return true;
        }
        if fast2 {
            self.grid_step = self.fast_grid_2;
            return true;
        }
        false
    }

    fn handle_fit_keys(&mut self, ui: &egui::Ui) {
        let (home, objects) = ui.input(|input| {
            let home = input.key_pressed(egui::Key::Home);
            (home, home && input.modifiers.command)
        });
        if objects {
            self.fit_view(FitTarget::Objects);
            return;
        }
        if home {
            self.fit_view(FitTarget::All);
        }
    }

    pub(crate) fn handle_history_keys(&mut self, ui: &egui::Ui) {
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

    pub(crate) fn handle_clipboard_keys(&mut self, ui: &egui::Ui) {
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

    pub(crate) fn handle_file_keys(&mut self, ui: &egui::Ui) {
        let save = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S));
        let open = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::O));
        if save {
            self.save_project();
        }
        if open {
            self.open_project();
        }
    }

    fn handle_tool_keys(&mut self, ui: &egui::Ui) {
        let tool = ui.input(|input| {
            if input.modifiers.command || input.modifiers.alt || input.modifiers.ctrl {
                return None;
            }
            tool_shortcut(input)
        });
        if let Some(tool) = tool {
            self.set_tool(tool);
        }
    }

    fn handle_panel_keys(&mut self, ui: &egui::Ui) {
        let (library, inspector, legend) = ui.input(|input| {
            let plain = !input.modifiers.command && !input.modifiers.alt && !input.modifiers.ctrl;
            (
                plain && input.key_pressed(egui::Key::L),
                plain && input.key_pressed(egui::Key::I),
                plain && input.key_pressed(egui::Key::K),
            )
        });
        if library {
            self.library_open = !self.library_open;
        }
        if inspector {
            self.inspector_open = !self.inspector_open;
        }
        if legend {
            self.legend_open = !self.legend_open;
        }
    }

    pub(crate) fn handle_edit_keys(&mut self, ui: &egui::Ui) {
        self.handle_transform_keys(ui);
        self.handle_wire_keys(ui);
        if ui.input(|input| input.key_pressed(egui::Key::Delete)) {
            self.delete_selection();
        }
    }

    pub(crate) fn handle_transform_keys(&mut self, ui: &egui::Ui) {
        if ui.input(|input| input.key_pressed(egui::Key::R)) {
            self.rotate();
        }
        if ui.input(|input| input.key_pressed(egui::Key::M)) {
            self.mirror();
        }
    }

    pub(crate) fn handle_wire_keys(&mut self, ui: &egui::Ui) {
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

    pub(crate) fn handle_nudge_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused || !ui.input(|input| input.modifiers.shift) {
            return;
        }
        let (dx, dy) = nudge_delta(ui);
        if dx == 0 && dy == 0 {
            return;
        }
        self.nudge_selection(dx, dy);
    }
}

fn tool_shortcut(input: &egui::InputState) -> Option<Tool> {
    if input.key_pressed(egui::Key::V) {
        return Some(Tool::Select);
    }
    if input.key_pressed(egui::Key::P) {
        return Some(Tool::Place);
    }
    if input.key_pressed(egui::Key::W) {
        return Some(Tool::Wire);
    }
    if input.key_pressed(egui::Key::G) {
        return Some(Tool::Probe);
    }
    None
}

fn cursor_delta(ui: &egui::Ui, step: i32) -> (i32, i32) {
    let mut delta = (0, 0);
    ui.input(|input| {
        delta.0 = axis_delta(input, egui::Key::ArrowLeft, egui::Key::ArrowRight, step);
        delta.1 = axis_delta(input, egui::Key::ArrowUp, egui::Key::ArrowDown, step);
    });
    delta
}
