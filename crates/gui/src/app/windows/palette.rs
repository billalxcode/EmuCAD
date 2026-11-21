use super::*;

impl GuiApp {
    pub(crate) fn palette_shortcut(&mut self, ui: &egui::Ui) {
        if self.conflict {
            self.palette.close();
            return;
        }
        let toggle = ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::K));
        if toggle {
            self.palette.toggle();
        }
    }

    pub(crate) fn command_palette(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn palette_state(&self) -> (bool, Vec<Command>) {
        let running = self.sim_status.running;
        let matches = crate::palette::matching(&self.palette.query, running);
        (running, matches)
    }

    pub(crate) fn palette_contents(
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

    pub(crate) fn palette_input(&mut self, ui: &mut egui::Ui, count: usize) {
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

    pub(crate) fn palette_keys(&mut self, ui: &egui::Ui, count: usize) {
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

    pub(crate) fn palette_submit(
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

    pub(crate) fn run_command(&mut self, command: Command) {
        self.palette.close();
        if self.run_edit_command(command) {
            return;
        }
        if self.run_layout_command(command) {
            return;
        }
        self.run_view_command(command);
    }

    pub(crate) fn run_layout_command(&mut self, command: Command) -> bool {
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

    pub(crate) fn run_edit_command(&mut self, command: Command) -> bool {
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

    pub(crate) fn run_view_command(&mut self, command: Command) {
        match command {
            Command::ToggleGrid => self.grid_visible = !self.grid_visible,
            Command::ToggleMinimap => self.minimap_open = !self.minimap_open,
            Command::ToggleLibrary => self.library_open = !self.library_open,
            Command::ToggleInspector => self.inspector_open = !self.inspector_open,
            Command::ToggleLegend => self.legend_open = !self.legend_open,
            Command::ToggleWaveform => self.waveform_open = !self.waveform_open,
            Command::ToggleUart => self.uart_open = !self.uart_open,
            Command::ToggleLog => self.log_open = !self.log_open,
            Command::ZoomReset => self.zoom_to(1.0),
            Command::ZoomUndo => {
                self.camera.undo_zoom();
            }
            Command::SelectTool => self.set_tool(Tool::Select),
            Command::WireTool => self.set_tool(Tool::Wire),
            _ => {}
        }
    }
}

impl GuiApp {
    pub(crate) fn set_tool(&mut self, tool: Tool) {
        if self.tool == tool {
            return;
        }
        self.tool = tool;
        self.wire_points.clear();
        self.box_start = None;
        self.box_current = None;
        self.dragging = None;
        self.dragging_segment = None;
    }
}
