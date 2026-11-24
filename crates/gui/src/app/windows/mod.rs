use super::*;

impl GuiApp {
    pub(crate) fn legend_window(&mut self, ui: &mut egui::Ui) {
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
}

mod context;
mod grid_settings;
mod library;
mod modals;
mod palette;
mod panels;
