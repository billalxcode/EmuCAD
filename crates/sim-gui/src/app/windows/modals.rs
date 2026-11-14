use super::*;

impl GuiApp {
    pub(crate) fn delete_modal(&mut self, ui: &mut egui::Ui) {
        if !self.pending_delete || self.conflict {
            return;
        }
        let mut action = None;
        let count = self.selection.len();
        egui::Modal::new(egui::Id::unique("delete-confirm")).show(ui.ctx(), |ui| {
            delete_dialog(ui, count, &mut action);
        });
        self.apply_delete_action(action);
    }

    pub(crate) fn apply_delete_action(&mut self, action: Option<bool>) {
        match action {
            Some(true) => {
                self.pending_delete = false;
                self.delete_selection_now();
            }
            Some(false) => self.pending_delete = false,
            None => {}
        }
    }

    pub(crate) fn conflict_modal(&mut self, ui: &mut egui::Ui) {
        if !self.conflict {
            return;
        }
        let path = self.current_path.clone().unwrap_or_default();
        let mut action = None;
        egui::Modal::new(egui::Id::unique("file-conflict")).show(ui.ctx(), |ui| {
            conflict_contents(ui, &path, &mut action);
        });
        if let Some(action) = action {
            self.apply_conflict(action);
        }
    }

    pub(crate) fn apply_conflict(&mut self, action: ConflictAction) {
        match action {
            ConflictAction::Reload => self.reload_from_disk(),
            ConflictAction::SaveMine => self.save_project_now(),
        }
    }

    pub(crate) fn reload_from_disk(&mut self) {
        let Some(path) = self.current_path.clone() else {
            self.conflict = false;
            return;
        };
        match persist::load(&path, self.library) {
            Ok(project) => {
                self.apply_project(project, path);
                self.message = "Reloaded".to_string();
            }
            Err(error) => self.message = error,
        }
    }
}
