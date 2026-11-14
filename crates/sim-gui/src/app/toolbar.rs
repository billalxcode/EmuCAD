use super::*;

impl GuiApp {
    pub(super) fn windows(&mut self, ui: &mut egui::Ui) {
        self.library_window(ui);
        self.inspector_window(ui);
        self.legend_window(ui);
        self.waveform_window(ui);
        self.analog_window(ui);
        self.uart_window(ui);
        self.log_window(ui);
        self.show_context_menu(ui);
        self.command_palette(ui);
        self.delete_modal(ui);
        self.show_toast(ui);
        self.conflict_modal(ui);
    }
    pub(super) fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                self.toolbar_tools(ui);
                ui.separator();
                self.toolbar_view(ui);
                if !self.message.is_empty() {
                    ui.separator();
                    ui.label(&self.message);
                }
            });
        });
    }

    pub(super) fn toolbar_tools(&mut self, ui: &mut egui::Ui) {
        ui.strong("EmuCAD");
        ui.separator();
        self.file_buttons(ui);
        self.edit_buttons(ui);
        ui.separator();
        self.sim_buttons(ui);
        ui.separator();
        self.tool_buttons(ui);
        ui.separator();
        self.sheet_buttons(ui);
    }

    pub(super) fn edit_buttons(&mut self, ui: &mut egui::Ui) {
        let undo = ui
            .add_enabled(self.history.can_undo(), icon_button("<-", "Undo"))
            .clicked();
        let redo = ui
            .add_enabled(self.history.can_redo(), icon_button("->", "Redo"))
            .clicked();
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    pub(super) fn file_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.add(icon_button("[/]", "Open")).clicked() {
            self.open_project();
        }
        if ui.add(icon_button("[S]", "Save")).clicked() {
            self.save_project();
        }
        if ui.add(icon_button("[P]", "PNG")).clicked() {
            self.export_png(ui.ctx());
        }
        self.recent_menu(ui);
    }

    pub(super) fn recent_menu(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        ui.menu_button("Recent", |ui| {
            if self.config.recent_files.is_empty() {
                ui.label("No recent files");
            }
            for path in &self.config.recent_files {
                let label = file_name(path);
                let hint = path.display().to_string();
                if ui.button(label).on_hover_text(hint).clicked() {
                    chosen = Some(path.clone());
                    ui.close();
                }
            }
        });
        if let Some(path) = chosen {
            self.open_path(path);
        }
    }

    pub(super) fn sim_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.add(icon_button("[H]", "Load HEX")).clicked() {
            self.open_hex();
        }
        self.run_buttons(ui);
        ui.checkbox(&mut self.waveform_open, "Wave");
        ui.checkbox(&mut self.analog_open, "Analog");
        ui.checkbox(&mut self.uart_open, "UART");
        self.sample_control(ui);
    }

    pub(super) fn run_buttons(&mut self, ui: &mut egui::Ui) {
        let loaded = self.sim_status.loaded;
        let running = self.sim_status.running;
        let run = ui.add_enabled(loaded, run_button_label(running)).clicked();
        let step = ui
            .add_enabled(loaded && !running, icon_button(">|", "Step"))
            .clicked();
        let reset = ui.add_enabled(loaded, icon_button("[]", "Reset")).clicked();
        self.apply_run_actions(run, step, reset);
    }

    pub(super) fn apply_run_actions(&mut self, run: bool, step: bool, reset: bool) {
        if run {
            self.toggle_run();
        }
        if step {
            self.send_sim(SimAction::Step);
        }
        if reset {
            self.send_sim(SimAction::Reset);
        }
    }

    pub(super) fn sample_control(&mut self, ui: &mut egui::Ui) {
        let current = self.sample_rate();
        let every = sample_combo(ui, current);
        if every == current {
            return;
        }
        let Ok(every) = crate::error::validate_sample_every(every) else {
            return;
        };
        self.sample_every = every;
        if let Some(handle) = &self.sim {
            let _ = handle.set_sample_every(every);
        }
    }

    pub(super) fn sample_rate(&self) -> u64 {
        if self.sample_every == 0 {
            sim::DEFAULT_SAMPLE_EVERY
        } else {
            self.sample_every
        }
    }

    pub(super) fn add_pb5_probe(&mut self) -> bool {
        self.add_probe(pb5_probe_spec())
    }

    pub(super) fn add_led_probe(&mut self) -> bool {
        self.add_probe(led_probe_spec())
    }

    pub(super) fn add_probe(&mut self, spec: ProbeSpec) -> bool {
        if self.probes.contains(spec.source) {
            return false;
        }
        self.probes.add(spec)
    }

    pub(super) fn open_pb5_probe(&mut self) {
        self.add_pb5_probe();
        self.analog_open = true;
    }

    pub(super) fn open_led_probe(&mut self) {
        self.add_led_probe();
        self.analog_open = true;
    }
}
