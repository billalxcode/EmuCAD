use super::*;
use crate::icons::IconKind;

impl GuiApp {
    pub(super) fn windows(&mut self, ui: &mut egui::Ui) {
        self.library_window(ui);
        self.inspector_window(ui);
        self.legend_window(ui);
        self.waveform_window(ui);
        self.analog_window(ui);
        self.uart_window(ui);
        self.log_window(ui);
        self.simulation_window(ui);
        self.transient_window(ui);
        self.grid_settings_window(ui);
        self.show_context_menu(ui);
        self.command_palette(ui);
        self.delete_modal(ui);
        self.show_toast(ui);
        self.conflict_modal(ui);
    }
    pub(crate) fn toolbar(&mut self, ui: &mut egui::Ui) {
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
        ui.label(egui::RichText::new("EmuCAD").font(crate::design::bold(16.0)));
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
            .add_enabled(
                self.history.can_undo(),
                icon_button(IconKind::Undo, "Undo").shortcut("Ctrl+Z"),
            )
            .clicked();
        let redo = ui
            .add_enabled(
                self.history.can_redo(),
                icon_button(IconKind::Redo, "Redo").shortcut("Ctrl+Shift+Z"),
            )
            .clicked();
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
    }

    pub(super) fn file_buttons(&mut self, ui: &mut egui::Ui) {
        let open = icon_button(IconKind::Open, "Open").shortcut("Ctrl+O");
        if ui.add(open).clicked() {
            self.open_project();
        }
        let save = icon_button(IconKind::Save, "Save").shortcut("Ctrl+S");
        if ui.add(save).clicked() {
            self.save_project();
        }
        if ui.add(icon_button(IconKind::Export, "PNG")).clicked() {
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
                let button = icon_button(IconKind::FileText, label).tooltip(hint);
                if ui.add(button).clicked() {
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
        if ui.add(icon_button(IconKind::LoadHex, "Load HEX")).clicked() {
            self.open_hex();
        }
        if ui
            .add(icon_button(IconKind::Netlist, "Check netlist"))
            .clicked()
        {
            self.check_netlist();
        }
        if ui
            .add_enabled(!self.analog_busy, icon_button(IconKind::Dc, "DC op"))
            .clicked()
        {
            self.solve_dc();
        }
        if ui
            .add_enabled(
                !self.analog_busy,
                icon_button(IconKind::Transient, "Transient"),
            )
            .clicked()
        {
            self.run_transient();
        }
        if self.analog_busy {
            ui.label("Solving");
        }
        self.run_buttons(ui);
        ui.checkbox(&mut self.simulation_open, "Sim");
        ui.checkbox(&mut self.transient_open, "Transient");
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
            .add_enabled(loaded && !running, icon_button(IconKind::Step, "Step"))
            .clicked();
        let reset = ui
            .add_enabled(loaded, icon_button(IconKind::Reset, "Reset"))
            .clicked();
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
