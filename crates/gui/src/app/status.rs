use super::*;
use crate::icons::IconKind;

fn status_label(ui: &mut egui::Ui, icon: IconKind, text: impl Into<String>) {
    ui.horizontal(|ui| {
        icon_slot(ui, icon);
        ui.label(text.into());
    });
}

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
            let fps = self.fps_text(ui);
            status_label(ui, IconKind::Dc, fps);
        });
    }

    pub(super) fn view_status(&self, ui: &mut egui::Ui) {
        status_label(ui, IconKind::Crosshair, self.pointer_text());
        ui.separator();
        status_label(ui, IconKind::Ruler, self.delta_text());
        ui.separator();
        let zoom = format!("Zoom {:.0}%", self.camera.zoom * 100.0);
        status_label(ui, IconKind::ZoomIn, zoom);
        ui.separator();
        let grid = format!("Grid x{}", self.grid_step.label());
        status_label(ui, IconKind::Grid, grid);
        ui.separator();
        status_label(ui, IconKind::Settings, self.snap_status());
        ui.separator();
        status_label(ui, IconKind::Ruler, self.units.label());
    }

    pub(super) fn delta_text(&self) -> String {
        let Some(world) = self.pointer_world else {
            return "dx=- dy=- dist=-".to_string();
        };
        let point = self.snap_world(world).point;
        let dx = point.x - self.local_origin.x;
        let dy = point.y - self.local_origin.y;
        let distance = ((dx * dx + dy * dy) as f64).sqrt();
        format!(
            "dx {} dy {} dist {}",
            self.units.format_coord(dx),
            self.units.format_coord(dy),
            self.units.format_distance(distance)
        )
    }

    pub(super) fn snap_status(&self) -> String {
        let mode = if !self.grid_snap_enabled() {
            "off"
        } else if self.snap_objects {
            "grid+objects"
        } else {
            "grid"
        };
        let source = self
            .pointer_world
            .map(|world| self.snap_world(world).kind.label());
        match source {
            Some(source) => format!("Snap {mode} ({source})"),
            None => format!("Snap {mode}"),
        }
    }

    pub(super) fn edit_status(&self, ui: &mut egui::Ui) {
        ui.label(format!("Tool: {}", self.tool.label()));
        ui.separator();
        let selection = self.selection.len();
        ui.label(format!("Sel {selection}"));
        ui.separator();
        ui.label(format!("Parts {}", self.document.components().len()));
        ui.separator();
        self.dangling_status(ui);
    }

    fn dangling_status(&self, ui: &mut egui::Ui) {
        let count = self.document.dangling_endpoints(self.library).len();
        if count == 0 {
            status_label(ui, IconKind::Check, "Dangling 0");
            return;
        }
        let color = crate::design::palette_for(ui.ctx()).danger;
        ui.horizontal(|ui| {
            icon_slot(ui, IconKind::Alert);
            ui.colored_label(color, format!("Dangling {count}"));
        });
    }

    pub(super) fn sim_status_labels(&self, ui: &mut egui::Ui) {
        status_label(ui, IconKind::Cpu, self.sim_text());
        ui.separator();
        status_label(ui, IconKind::Diode, self.led_text());
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
            Some(world) => {
                let point = self.snap_world(world).point;
                format!(
                    "X {} Y {}",
                    self.units.format_coord(point.x),
                    self.units.format_coord(point.y)
                )
            }
            None => "X - Y -".to_string(),
        }
    }

    pub(super) fn fps_text(&self, ui: &egui::Ui) -> String {
        let dt = ui.ctx().input(|input| input.stable_dt).max(0.0001);
        format!("FPS {:.0}", 1.0 / dt)
    }
}
