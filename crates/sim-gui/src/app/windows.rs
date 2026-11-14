use super::*;

impl GuiApp {
    pub(super) fn legend_window(&mut self, ui: &mut egui::Ui) {
        if !self.legend_open {
            return;
        }
        let mut open = self.legend_open;
        egui::Window::new("Net legend")
            .open(&mut open)
            .default_width(190.0)
            .show(ui.ctx(), |ui| {
                show_net_legend(ui);
            });
        self.legend_open = open;
    }

    pub(super) fn show_context_menu(&mut self, ui: &mut egui::Ui) {
        let Some(position) = self.context_menu else {
            return;
        };
        if self.conflict || self.palette.open {
            self.context_menu = None;
            return;
        }
        let (action, response) = self.context_area(ui, position);
        if context_closes(ui, &response, action.is_some()) {
            self.context_menu = None;
        }
        if let Some(action) = action {
            self.context_menu = None;
            self.run_context_action(action);
        }
    }

    pub(super) fn context_area(
        &self,
        ui: &egui::Ui,
        position: egui::Pos2,
    ) -> (Option<ContextAction>, egui::Response) {
        show_context_area(
            ui.ctx(),
            position,
            &self.selection,
            self.clipboard.is_empty(),
            self.context_net_state(),
        )
    }

    pub(super) fn run_context_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::Copy => self.copy_selection(),
            ContextAction::Paste => self.paste_selection(),
            ContextAction::Duplicate => self.duplicate_selection(),
            ContextAction::Delete => self.delete_selection(),
            ContextAction::Rotate => self.rotate(),
            ContextAction::Mirror => self.mirror(),
            ContextAction::Properties => self.inspector_open = true,
            ContextAction::Graph => self.waveform_open = true,
            ContextAction::ProbePb5 => self.open_pb5_probe(),
            ContextAction::ProbeLed => self.open_led_probe(),
            ContextAction::SetClass(class) => self.set_wire_class(class),
            ContextAction::SetColor(color) => self.set_wire_color_override(color),
            ContextAction::SetLabel(label) => self.set_wire_label(label),
            ContextAction::SelectTool => self.set_tool(Tool::Select),
            ContextAction::WireTool => self.set_tool(Tool::Wire),
        }
    }

    pub(super) fn palette_shortcut(&mut self, ui: &egui::Ui) {
        if self.conflict {
            self.palette.close();
            return;
        }
        let toggle = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::K));
        if toggle {
            self.palette.toggle();
        }
    }

    pub(super) fn command_palette(&mut self, ui: &mut egui::Ui) {
        if !self.palette.open {
            return;
        }
        if self.conflict {
            self.palette.close();
            return;
        }
        let (running, matches) = self.palette_state();
        let mut chosen = None;
        let response = egui::Modal::new(egui::Id::unique("command-palette")).show(ui.ctx(), |ui| {
            self.palette_contents(ui, running, &matches, &mut chosen);
        });
        if response.backdrop_response.clicked() {
            self.palette.close();
        }
        if let Some(command) = chosen {
            self.run_command(command);
        }
    }

    pub(super) fn palette_state(&self) -> (bool, Vec<Command>) {
        let running = self.sim_status.running;
        let matches = palette::matching(&self.palette.query, running);
        (running, matches)
    }

    pub(super) fn palette_contents(
        &mut self,
        ui: &mut egui::Ui,
        running: bool,
        matches: &[Command],
        chosen: &mut Option<Command>,
    ) {
        ui.set_width(460.0);
        self.palette_input(ui, matches.len());
        ui.separator();
        palette_list(ui, running, matches, self.palette.selected, chosen);
        self.palette_submit(ui, matches, chosen);
    }

    pub(super) fn palette_input(&mut self, ui: &mut egui::Ui, count: usize) {
        self.palette_keys(ui, count);
        let edit = ui.text_edit_singleline(&mut self.palette.query);
        if self.palette.focus {
            edit.request_focus();
            self.palette.focus = false;
        }
        if edit.changed() {
            self.palette.selected = 0;
        }
    }

    pub(super) fn palette_keys(&mut self, ui: &egui::Ui, count: usize) {
        let down =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up = ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        if down {
            self.palette.move_selection(count, 1);
        }
        if up {
            self.palette.move_selection(count, -1);
        }
    }

    pub(super) fn palette_submit(
        &mut self,
        ui: &egui::Ui,
        matches: &[Command],
        chosen: &mut Option<Command>,
    ) {
        let enter =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        if enter {
            *chosen = matches.get(self.palette.selected).copied();
        }
        let escape =
            ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if escape {
            self.palette.close();
        }
    }

    pub(super) fn run_command(&mut self, command: Command) {
        self.palette.close();
        if self.run_edit_command(command) {
            return;
        }
        if self.run_layout_command(command) {
            return;
        }
        self.run_view_command(command);
    }

    pub(super) fn run_layout_command(&mut self, command: Command) -> bool {
        match command {
            Command::AlignLeft => self.align_selection(Align::Left),
            Command::AlignRight => self.align_selection(Align::Right),
            Command::AlignTop => self.align_selection(Align::Top),
            Command::AlignBottom => self.align_selection(Align::Bottom),
            Command::AlignCenterX => self.align_selection(Align::CenterX),
            Command::AlignCenterY => self.align_selection(Align::CenterY),
            Command::DistributeHorizontal => self.distribute_selection(Distribute::Horizontal),
            Command::DistributeVertical => self.distribute_selection(Distribute::Vertical),
            _ => return false,
        }
        true
    }

    pub(super) fn run_edit_command(&mut self, command: Command) -> bool {
        match command {
            Command::Save => self.save_project(),
            Command::Open => self.open_project(),
            Command::LoadHex => self.open_hex(),
            Command::RunPause => self.toggle_run(),
            Command::Step => self.send_sim(SimAction::Step),
            Command::Reset => self.send_sim(SimAction::Reset),
            Command::Undo => self.undo(),
            Command::Redo => self.redo(),
            Command::Copy => self.copy_selection(),
            Command::Paste => self.paste_selection(),
            Command::Duplicate => self.duplicate_selection(),
            Command::Delete => self.delete_selection(),
            _ => return false,
        }
        true
    }

    pub(super) fn run_view_command(&mut self, command: Command) {
        match command {
            Command::ToggleGrid => self.grid_visible = !self.grid_visible,
            Command::ToggleMinimap => self.minimap_open = !self.minimap_open,
            Command::ToggleLibrary => self.library_open = !self.library_open,
            Command::ToggleInspector => self.inspector_open = !self.inspector_open,
            Command::ToggleLegend => self.legend_open = !self.legend_open,
            Command::ToggleWaveform => self.waveform_open = !self.waveform_open,
            Command::ToggleUart => self.uart_open = !self.uart_open,
            Command::ToggleLog => self.log_open = !self.log_open,
            Command::ZoomReset => self.camera.zoom = 1.0,
            Command::SelectTool => self.set_tool(Tool::Select),
            Command::WireTool => self.set_tool(Tool::Wire),
            _ => {}
        }
    }

    pub(super) fn set_tool(&mut self, tool: Tool) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.wire_points.clear();
    }

    pub(super) fn library_window(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn show_library(
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

    pub(super) fn library_contents(&self, ui: &mut egui::Ui, state: &mut LibraryState) {
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

    pub(super) fn finish_library_choice(
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

    pub(super) fn recent_buttons(&self, ui: &mut egui::Ui) -> Option<&'static ComponentDefinition> {
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

    pub(super) fn library_filters(
        &self,
        ui: &mut egui::Ui,
        search: &mut String,
        category: &mut Option<Category>,
    ) {
        ui.text_edit_singleline(search);
        ui.horizontal_wrapped(|ui| category_buttons(ui, category));
    }

    pub(super) fn inspector_window(&mut self, ui: &mut egui::Ui) {
        let mut open = self.inspector_open;
        let mut action = None;
        egui::Window::new("Inspector")
            .open(&mut open)
            .default_width(260.0)
            .show(ui.ctx(), |ui| {
                self.inspector_contents(ui, &mut action);
            });
        self.inspector_open = open;
        if let Some(action) = action {
            self.apply_inspector(action);
        }
    }

    pub(super) fn log_window(&mut self, ui: &mut egui::Ui) {
        if !self.log_open {
            return;
        }
        let mut open = self.log_open;
        let mut clear = false;
        egui::Window::new("Log")
            .open(&mut open)
            .default_size([460.0, 220.0])
            .show(ui.ctx(), |ui| {
                show_log_contents(ui, &self.log, &mut clear);
            });
        self.log_open = open;
        if clear {
            self.log.clear();
        }
    }

    pub(super) fn uart_window(&mut self, ui: &mut egui::Ui) {
        if !self.uart_open {
            return;
        }
        let mut open = self.uart_open;
        let mut actions = (false, false);
        let mut input = self.uart_input.clone();
        egui::Window::new("UART monitor")
            .open(&mut open)
            .default_size([420.0, 260.0])
            .show(ui.ctx(), |ui| {
                show_uart_contents(ui, &self.uart, &mut input, &mut actions.0, &mut actions.1);
            });
        self.uart_open = open;
        self.finish_uart(input, actions);
    }

    pub(super) fn finish_uart(&mut self, input: String, actions: (bool, bool)) {
        self.uart_input = input;
        if actions.0 {
            self.uart.clear();
        }
        if actions.1 {
            self.send_uart_input();
        }
    }

    pub(super) fn send_uart_input(&mut self) {
        if self.uart_input.is_empty() {
            return;
        }
        let data = self.uart_input.as_bytes().to_vec();
        let Some(sim) = self.sim.as_ref() else {
            self.message = "Simulation unavailable".to_string();
            return;
        };
        match sim.send_uart(data) {
            Ok(()) => {
                self.uart_input.clear();
                self.message = "UART RX sent".to_string();
            }
            Err(error) => self.message = error,
        }
    }

    pub(super) fn waveform_window(&mut self, ui: &mut egui::Ui) {
        if !self.waveform_open {
            return;
        }
        let mut open = self.waveform_open;
        let status = self.sim_status;
        egui::Window::new("Digital graph")
            .open(&mut open)
            .default_size([520.0, 220.0])
            .show(ui.ctx(), |ui| {
                show_digital_graph(ui, &self.waveform, status);
            });
        self.waveform_open = open;
    }

    pub(super) fn analog_window(&mut self, ui: &mut egui::Ui) {
        if !self.analog_open {
            return;
        }
        let mut open = self.analog_open;
        let status = self.sim_status;
        let probes = &self.probes;
        let view = &mut self.plot_view;
        egui::Window::new("Analog graph")
            .open(&mut open)
            .default_size([560.0, 300.0])
            .show(ui.ctx(), |ui| {
                show_analog_graph(ui, probes, view, status);
            });
        self.analog_open = open;
    }

    pub(super) fn delete_modal(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn apply_delete_action(&mut self, action: Option<bool>) {
        match action {
            Some(true) => {
                self.pending_delete = false;
                self.delete_selection_now();
            }
            Some(false) => self.pending_delete = false,
            None => {}
        }
    }

    pub(super) fn conflict_modal(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn apply_conflict(&mut self, action: ConflictAction) {
        match action {
            ConflictAction::Reload => self.reload_from_disk(),
            ConflictAction::SaveMine => self.save_project_now(),
        }
    }

    pub(super) fn reload_from_disk(&mut self) {
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
