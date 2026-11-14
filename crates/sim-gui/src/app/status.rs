use super::*;

impl GuiApp {
    pub(super) fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status").show(ui, |ui| {
            self.status_contents(ui);
        });
    }

    pub(super) fn status_contents(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.view_status(ui);
            ui.separator();
            self.edit_status(ui);
            ui.separator();
            self.sim_status_labels(ui);
            ui.separator();
            ui.label(self.fps_text(ui));
        });
    }

    pub(super) fn view_status(&self, ui: &mut egui::Ui) {
        ui.label(self.pointer_text());
        ui.separator();
        ui.label(format!("Zoom {:.0}%", self.camera.zoom * 100.0));
        ui.separator();
        ui.label(format!("Grid {:.0}px", self.camera.cell_px));
        ui.separator();
        ui.label("Snap grid");
    }

    pub(super) fn edit_status(&self, ui: &mut egui::Ui) {
        ui.label(format!("Tool: {}", self.tool.label()));
        ui.separator();
        let selection = self.selection.len();
        ui.label(format!("Sel {selection}"));
        ui.separator();
        ui.label(format!("Parts {}", self.document.components().len()));
    }

    pub(super) fn sim_status_labels(&self, ui: &mut egui::Ui) {
        ui.label(self.sim_text());
        ui.separator();
        ui.label(self.led_text());
    }

    pub(super) fn sim_text(&self) -> String {
        if !self.sim_status.loaded {
            return "Sim idle".to_string();
        }
        let state = if self.sim_status.running {
            "running"
        } else {
            "paused"
        };
        format!("Sim {} cycles ({state})", self.sim_status.cycles)
    }

    pub(super) fn led_text(&self) -> String {
        if !self.sim_status.loaded {
            return "PB5 -".to_string();
        }
        format!(
            "PB5 {} {} mV  LED {} {:.1} mA",
            high_low(self.sim_status.pb5_high),
            self.sim_status.pb5_millivolts,
            on_off(self.sim_status.led_on),
            self.sim_status.led_microamps as f32 / 1000.0
        )
    }

    pub(super) fn pointer_text(&self) -> String {
        match self.pointer_world {
            Some(point) => format!("x={:.1} y={:.1}", point.x, point.y),
            None => "x=- y=-".to_string(),
        }
    }

    pub(super) fn fps_text(&self, ui: &egui::Ui) -> String {
        let dt = ui.ctx().input(|input| input.stable_dt).max(0.0001);
        format!("FPS {:.0}", 1.0 / dt)
    }
}
