use super::*;

impl GuiApp {
    pub(crate) fn grid_settings_window(&mut self, ui: &mut egui::Ui) {
        if !self.grid_settings_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Grid settings")
            .open(&mut open)
            .default_width(340.0)
            .show(ui.ctx(), |ui| {
                self.grid_settings_contents(ui);
            });
        self.grid_settings_open = open;
    }

    fn grid_settings_contents(&mut self, ui: &mut egui::Ui) {
        self.grid_settings_grid(ui);
        ui.separator();
        self.grid_settings_display(ui);
        ui.separator();
        self.grid_settings_units(ui);
    }

    fn grid_settings_grid(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.grid_visible, "Show grid");
        ui.horizontal(|ui| {
            ui.label("Step");
            grid_step_combo(ui, "settings_step", &mut self.grid_step);
        });
        ui.horizontal(|ui| {
            ui.label("Fast grid 1");
            grid_step_combo(ui, "settings_fast_1", &mut self.fast_grid_1);
        });
        ui.horizontal(|ui| {
            ui.label("Fast grid 2");
            grid_step_combo(ui, "settings_fast_2", &mut self.fast_grid_2);
        });
    }

    fn grid_settings_display(&mut self, ui: &mut egui::Ui) {
        self.grid_settings_style(ui);
        ui.checkbox(&mut self.grid_axes, "Axes");
        ui.checkbox(&mut self.grid_rulers, "Rulers");
        self.grid_settings_metrics(ui);
    }

    fn grid_settings_style(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Style");
            for style in canvas::GridStyle::ALL {
                ui.radio_value(&mut self.grid_style, style, style.label());
            }
        });
        ui.horizontal(|ui| {
            ui.label("Crosshair");
            for mode in scene::CrosshairMode::ALL {
                ui.radio_value(&mut self.crosshair, mode, mode.label());
            }
        });
    }

    fn grid_settings_metrics(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Line width");
            ui.add(egui::DragValue::new(&mut self.grid_line_width).range(0.5..=10.0));
        });
        ui.horizontal(|ui| {
            ui.label("Min spacing");
            ui.add(egui::DragValue::new(&mut self.grid_min_spacing).range(5.0..=200.0));
        });
    }

    fn grid_settings_units(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Units");
            for units in Units::ALL {
                ui.radio_value(&mut self.units, units, units.label());
            }
        });
        ui.horizontal(|ui| {
            ui.label("Local origin (u)");
            ui.add(egui::DragValue::new(&mut self.local_origin.x));
            ui.add(egui::DragValue::new(&mut self.local_origin.y));
        });
    }
}

fn grid_step_combo(ui: &mut egui::Ui, id: &str, step: &mut GridStep) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(format!("x{}", step.label()))
        .show_ui(ui, |ui| {
            for option in GridStep::ALL {
                ui.selectable_value(step, option, format!("x{}", option.label()));
            }
        });
}
