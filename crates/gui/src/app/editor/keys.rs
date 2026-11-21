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
        self.handle_edit_keys(ui);
        self.handle_nudge_keys(ui);
        self.handle_view_keys(ui);
    }

    pub(crate) fn handle_view_keys(&mut self, ui: &egui::Ui) {
        if !self.canvas_focused {
            return;
        }
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
        if !self.canvas_focused {
            return;
        }
        let (dx, dy) = nudge_delta(ui);
        if dx == 0 && dy == 0 {
            return;
        }
        self.nudge_selection(dx, dy);
    }
}
