use super::*;

impl GuiApp {
    pub(crate) fn library_window(&mut self, ui: &mut egui::Ui) {
        let mut open = self.library_open;
        let mut state = LibraryState {
            search: self.search.clone(),
            category: self.category,
            chosen: None,
            hovered: None,
            dragged: None,
        };
        self.show_library(ui, &mut open, &mut state);
        self.library_open = open;
        self.search = state.search;
        self.category = state.category;
        self.finish_library_choice(state.dragged, state.chosen);
    }

    pub(crate) fn show_library(
        &self,
        ui: &mut egui::Ui,
        open: &mut bool,
        state: &mut LibraryState,
    ) {
        egui::Window::new("Library")
            .open(open)
            .default_width(300.0)
            .show(ui.ctx(), |ui| self.library_contents(ui, state));
    }

    pub(crate) fn library_contents(&self, ui: &mut egui::Ui, state: &mut LibraryState) {
        if let Some(definition) = self.recent_buttons(ui) {
            state.chosen = Some(definition);
        }
        self.library_filters(ui, &mut state.search, &mut state.category);
        ui.separator();
        if state.chosen.is_none() {
            state.chosen = library_list(
                ui,
                self.library,
                &state.search,
                state.category,
                &mut state.hovered,
                &mut state.dragged,
            );
        }
        if let Some(definition) = state.hovered {
            library_preview(ui, definition);
        }
    }

    pub(crate) fn finish_library_choice(
        &mut self,
        dragged: Option<&'static ComponentDefinition>,
        chosen: Option<&'static ComponentDefinition>,
    ) {
        if let Some(definition) = dragged {
            self.placing = Some(definition);
            self.drag_place = true;
            self.library_open = false;
            return;
        }
        if let Some(definition) = chosen {
            self.placing = Some(definition);
            self.library_open = false;
        }
    }

    pub(crate) fn recent_buttons(&self, ui: &mut egui::Ui) -> Option<&'static ComponentDefinition> {
        if self.recent_components.is_empty() {
            return None;
        }
        ui.label("Recent");
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            for definition in &self.recent_components {
                if ui.button(definition.name).clicked() {
                    chosen = Some(*definition);
                }
            }
        });
        ui.separator();
        chosen
    }

    pub(crate) fn library_filters(
        &self,
        ui: &mut egui::Ui,
        search: &mut String,
        category: &mut Option<Category>,
    ) {
        ui.text_edit_singleline(search);
        ui.horizontal_wrapped(|ui| category_buttons(ui, category));
    }
}
