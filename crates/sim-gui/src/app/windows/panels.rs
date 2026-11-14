use super::*;

impl GuiApp {
    pub(crate) fn inspector_window(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn log_window(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn uart_window(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn finish_uart(&mut self, input: String, actions: (bool, bool)) {
        self.uart_input = input;
        if actions.0 {
            self.uart.clear();
        }
        if actions.1 {
            self.send_uart_input();
        }
    }

    pub(crate) fn send_uart_input(&mut self) {
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

    pub(crate) fn waveform_window(&mut self, ui: &mut egui::Ui) {
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

    pub(crate) fn analog_window(&mut self, ui: &mut egui::Ui) {
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
}
