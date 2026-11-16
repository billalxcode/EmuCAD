use super::*;

impl GuiApp {
    pub(super) fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("EmuCAD project", &["json"])
            .pick_file()
        else {
            self.message = "Open cancelled".to_string();
            return;
        };
        self.open_path(path);
    }

    pub(super) fn open_path(&mut self, path: PathBuf) {
        if !self.flush_autosave() {
            return;
        }
        match persist::load(&path, self.library) {
            Ok(project) => {
                self.apply_project(project, path.clone());
                self.remember_recent(&path);
            }
            Err(error) => self.message = error,
        }
    }

    pub(super) fn save_project(&mut self) {
        if self.external_changed() {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
            return;
        }
        self.save_project_now();
    }

    pub(super) fn save_project_now(&mut self) {
        let path = self.current_path.clone().or_else(select_save_path);
        let Some(path) = path else {
            self.message = "Save cancelled".to_string();
            return;
        };
        match persist::save(&path, &self.document, &self.camera, self.grid_visible) {
            Ok(()) => self.after_save(path),
            Err(error) => self.message = error,
        }
    }

    pub(super) fn after_save(&mut self, path: PathBuf) {
        self.current_path = Some(path.clone());
        self.record_file_stamp(&path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Saved".to_string();
        self.remember_recent(&path);
    }

    pub(super) fn remember_recent(&mut self, path: &Path) {
        config::push_recent(&mut self.config, path);
        self.save_config();
    }

    pub(super) fn save_config(&mut self) {
        let Some(path) = self.config_path.as_deref() else {
            return;
        };
        if let Err(error) = config::save(path, &self.config) {
            self.message = error;
        }
    }

    pub(super) fn export_png(&mut self, ctx: &egui::Context) {
        let Some(canvas) = self.canvas_rect else {
            self.message = "Canvas is not ready".to_string();
            return;
        };
        let Some(path) = select_export_path() else {
            self.message = "PNG export cancelled".to_string();
            return;
        };
        let screen = ctx.content_rect();
        let callback = png_callback(screen, path, canvas, self.png_result.clone());
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
        self.message = "Exporting PNG".to_string();
    }

    pub(super) fn poll_png(&mut self) {
        let Ok(mut slot) = self.png_result.lock() else {
            return;
        };
        let Some(result) = slot.take() else {
            return;
        };
        self.message = match result {
            Ok(()) => "PNG saved".to_string(),
            Err(error) => GuiError::Export(error).to_string(),
        };
    }

    pub(super) fn update_toast(&mut self) {
        if self.message.is_empty() || self.message == self.last_toast {
            return;
        }
        self.last_toast = self.message.clone();
        self.log
            .push(format!("[{}] {}", self.sim_status.cycles, self.message));
        self.toast_until = Some(Instant::now() + TOAST_DURATION);
    }

    pub(super) fn show_toast(&mut self, ui: &egui::Ui) {
        let Some(until) = self.toast_until else {
            return;
        };
        let now = Instant::now();
        if now >= until {
            self.toast_until = None;
            return;
        }
        egui::Area::new(egui::Id::unique("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -44.0))
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(&self.message);
                });
            });
        ui.ctx().request_repaint_after(until - now);
    }

    pub(super) fn autosave(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let Some(remaining) = self.autosave_remaining(now) else {
            return;
        };
        if remaining.is_zero() {
            self.write_autosave(now);
            return;
        }
        ctx.request_repaint_after(remaining);
    }

    pub(super) fn autosave_remaining(&self, now: Instant) -> Option<Duration> {
        if self.conflict || !self.dirty || self.current_path.is_none() {
            return None;
        }
        let last = self.last_edit?;
        Some(AUTOSAVE_DELAY.saturating_sub(now.saturating_duration_since(last)))
    }

    pub(super) fn write_autosave(&mut self, now: Instant) {
        let Some(path) = self.current_path.clone() else {
            return;
        };
        if self.external_changed() {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
            return;
        }
        match persist::save(&path, &self.document, &self.camera, self.grid_visible) {
            Ok(()) => self.finish_autosave(&path),
            Err(error) => self.fail_autosave(now, error),
        }
    }

    pub(super) fn finish_autosave(&mut self, path: &Path) {
        self.record_file_stamp(path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Autosaved".to_string();
    }

    pub(super) fn fail_autosave(&mut self, now: Instant, error: String) {
        self.last_edit = Some(now);
        self.message = error;
    }

    pub(super) fn external_changed(&self) -> bool {
        let Some((path, stamp)) = self.watch_target() else {
            return false;
        };
        file_stamp(&path) != Some(stamp)
    }

    pub(super) fn mark_dirty(&mut self) {
        self.dirty = true;
        self.last_edit = Some(Instant::now());
        self.transient = None;
        self.analog_dc = None;
        self.analog_request = None;
        self.analog_busy = false;
    }

    pub(super) fn flush_autosave(&mut self) -> bool {
        if self.conflict {
            self.message = "Resolve file conflict before continuing".to_string();
            return false;
        }
        if !self.dirty {
            return true;
        }
        if self.current_path.is_none() {
            self.message = "Save project before opening another file".to_string();
            return false;
        }
        self.write_autosave(Instant::now());
        !self.dirty
    }

    pub(super) fn record_file_stamp(&mut self, path: &Path) {
        self.file_stamp = file_stamp(path);
        self.watch_at = Some(Instant::now());
        self.conflict = false;
    }

    pub(super) fn watch_file(&mut self, ctx: &egui::Context) {
        if self.conflict {
            return;
        }
        let Some((path, stamp)) = self.watch_target() else {
            return;
        };
        let now = Instant::now();
        let remaining = watch_remaining(self.watch_at, now);
        if !remaining.is_zero() {
            ctx.request_repaint_after(remaining);
            return;
        }
        self.watch_at = Some(now);
        ctx.request_repaint_after(WATCH_INTERVAL);
        if file_stamp(&path) != Some(stamp) {
            self.conflict = true;
            self.message = "File changed on disk".to_string();
        }
    }

    pub(super) fn watch_target(&self) -> Option<(PathBuf, FileStamp)> {
        Some((self.current_path.clone()?, self.file_stamp?))
    }

    pub(super) fn apply_project(&mut self, project: persist::Project, path: PathBuf) {
        self.document = project.document;
        self.camera = project.camera;
        self.grid_visible = project.grid_visible;
        self.reset_transient();
        self.history.clear();
        self.initialized = true;
        self.current_path = Some(path.clone());
        self.record_file_stamp(&path);
        self.dirty = false;
        self.last_edit = None;
        self.message = "Loaded".to_string();
    }

    pub(super) fn open_hex(&mut self) {
        let Some(path) = pick_hex() else {
            self.message = "HEX open cancelled".to_string();
            return;
        };
        let name = file_name(&path);
        match sim::load_hex(&path).map_err(GuiError::Firmware) {
            Ok(words) => self.start_firmware(words, name),
            Err(error) => self.message = error.to_string(),
        }
    }

    pub(super) fn start_firmware(&mut self, words: Vec<u16>, name: String) {
        match self.load_sim(words) {
            Ok(()) => {
                self.waveform.clear();
                self.probes.clear_samples();
                self.uart.clear();
                self.uart_input.clear();
                self.waveform_open = true;
                self.message = format!("Firmware {name}");
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    pub(super) fn load_sim(&self, words: Vec<u16>) -> GuiResult<()> {
        let Some(sim) = self.sim.as_ref() else {
            return Err(GuiError::Simulation("simulation unavailable".to_string()));
        };
        sim.load(words).map_err(GuiError::Simulation)
    }

    pub(super) fn toggle_run(&mut self) {
        let action = if self.sim_status.running {
            SimAction::Pause
        } else {
            SimAction::Run
        };
        self.send_sim(action);
    }

    pub(super) fn send_sim(&mut self, action: SimAction) {
        let Some(sim) = self.sim.as_ref() else {
            self.message = "Simulation unavailable".to_string();
            return;
        };
        if let Err(error) = sim_command(sim, action) {
            self.message = error;
        }
        if action == SimAction::Reset {
            self.clear_sim_history();
        }
    }

    pub(super) fn clear_sim_history(&mut self) {
        self.waveform.clear();
        self.probes.clear_samples();
        self.uart.clear();
        self.uart_input.clear();
    }

    pub(super) fn poll_sim(&mut self) {
        let events = self.sim.as_ref().map(SimHandle::poll).unwrap_or_default();
        for event in events {
            self.apply_sim_event(event);
        }
    }

    pub(super) fn apply_sim_event(&mut self, event: Event) {
        match event {
            Event::Status(status) => self.sim_status = status,
            Event::Sample(sample) => self.push_sample(sample),
            Event::Uart(data) => self.uart.push(&data),
            Event::UartCleared => self.uart.clear(),
            Event::Error(error) => {
                self.sim_status.running = false;
                self.message = error;
            }
        }
    }

    pub(super) fn push_sample(&mut self, sample: Sample) {
        self.waveform.push(sample);
        let provider = SampleProvider { sample };
        self.probes.sample_from(&provider, sample.cycle);
    }
}
