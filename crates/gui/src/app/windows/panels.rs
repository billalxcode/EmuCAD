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

    pub(crate) fn transient_window(&mut self, ui: &mut egui::Ui) {
        if !self.transient_open {
            return;
        }
        let mut open = self.transient_open;
        let trace = self.transient.as_ref();
        let previous = self.transient_settings;
        let settings = &mut self.transient_settings;
        let selected = &mut self.transient_selected;
        let selected_currents = &mut self.transient_selected_currents;
        egui::Window::new("Transient")
            .open(&mut open)
            .default_size([620.0, 340.0])
            .show(ui.ctx(), |ui| {
                if let Some(trace) = trace {
                    show_transient(ui, trace, settings, selected, selected_currents);
                } else {
                    show_transient_settings(ui, settings);
                    ui.label("Run transient to see node voltages");
                }
            });
        self.transient_open = open;
        if self.transient_settings != previous {
            self.mark_dirty();
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

impl GuiApp {
    pub(crate) fn simulation_window(&mut self, ui: &mut egui::Ui) {
        if !self.simulation_open {
            return;
        }
        let mut open = self.simulation_open;
        let mut run = false;
        let mut pause = false;
        let mut step = false;
        let mut reset = false;
        let mut open_transient = false;
        let status = self.netlist_status();
        let previous_domain = self.simulation_domain;
        let domain = &mut self.simulation_domain;
        let previous = self.analysis_kind;
        let kind = &mut self.analysis_kind;
        let dc = self.analog_dc.as_ref();
        let trace = self.transient.as_ref();
        let busy = self.analog_busy;
        let paused = self.analog_paused;
        let has_trace = self.transient.is_some();
        egui::Window::new("Simulation")
            .open(&mut open)
            .default_size([420.0, 320.0])
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    for option in [
                        crate::analysis::SimulationDomain::Analog,
                        crate::analysis::SimulationDomain::Avr,
                        crate::analysis::SimulationDomain::Mixed,
                    ] {
                        ui.selectable_value(domain, option, option.label());
                    }
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(kind, crate::analysis::AnalysisKind::Dc, "DC");
                    ui.selectable_value(
                        kind,
                        crate::analysis::AnalysisKind::Transient,
                        "Transient",
                    );
                });
                ui.label(format!(
                    "Domain: {}  Analysis: {}",
                    domain.label(),
                    kind.label()
                ));
                ui.label(status);
                if busy {
                    ui.label("Solving...");
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(!busy, egui::Button::new("Run")).clicked() {
                        run = true;
                    }
                    if ui
                        .add_enabled(busy && !paused, egui::Button::new("Pause"))
                        .clicked()
                    {
                        pause = true;
                    }
                    if ui
                        .add_enabled(busy && paused, egui::Button::new("Step"))
                        .clicked()
                    {
                        step = true;
                    }
                    if ui
                        .add_enabled(busy || has_trace, egui::Button::new("Reset"))
                        .clicked()
                    {
                        reset = true;
                    }
                });
                if *kind == crate::analysis::AnalysisKind::Dc {
                    show_dc_results(ui, dc);
                } else {
                    show_transient_summary(ui, trace, &mut open_transient);
                }
            });
        self.simulation_open = open;
        if self.analysis_kind != previous || self.simulation_domain != previous_domain {
            self.mark_dirty();
        }
        if run {
            self.run_analysis();
        }
        if pause {
            self.pause_transient();
        }
        if step {
            self.step_transient();
        }
        if reset {
            self.reset_transient_session();
        }
        if open_transient {
            self.transient_open = true;
        }
    }

    fn netlist_status(&self) -> String {
        match crate::netlist::extract(&self.document) {
            Ok(netlist) => format!(
                "Netlist OK: {} nodes, {} parts",
                netlist.nodes.len(),
                netlist.components.len()
            ),
            Err(error) => format!("Netlist error: {error}"),
        }
    }
}

fn show_dc_results(ui: &mut egui::Ui, dc: Option<&crate::analog::DcResultView>) {
    let Some(dc) = dc else {
        ui.label("No DC result");
        return;
    };
    ui.label(format!(
        "DC converged: {} iterations, residual {:.3e}",
        dc.iterations, dc.residual
    ));
    ui.label("Node voltages");
    show_dc_grid(ui, &dc.voltages);
    show_dc_currents(ui, &dc.currents);
}

fn show_dc_currents(ui: &mut egui::Ui, currents: &[(String, Option<f64>)]) {
    if currents.is_empty() {
        return;
    }
    ui.label("Branch currents");
    egui::ScrollArea::vertical()
        .max_height(120.0)
        .show(ui, |ui| {
            egui::Grid::new("dc_currents").striped(true).show(ui, |ui| {
                for (name, current) in currents {
                    show_dc_current(ui, name, *current);
                }
            });
        });
}

fn show_dc_current(ui: &mut egui::Ui, name: &str, current: Option<f64>) {
    ui.label(name);
    match current {
        Some(value) => ui.label(format!("{value:.6} A")),
        None => ui.label("n/a"),
    };
    ui.end_row();
}

fn show_dc_grid(ui: &mut egui::Ui, voltages: &[(String, f64)]) {
    egui::ScrollArea::vertical()
        .max_height(180.0)
        .show(ui, |ui| {
            egui::Grid::new("dc_results").striped(true).show(ui, |ui| {
                for (name, value) in voltages {
                    ui.label(name);
                    ui.label(format!("{value:.6} V"));
                    ui.end_row();
                }
            });
        });
}

fn show_transient_summary(
    ui: &mut egui::Ui,
    trace: Option<&crate::transient::TransientTrace>,
    open: &mut bool,
) {
    let Some(trace) = trace else {
        ui.label("No transient result");
        return;
    };
    ui.label(format!(
        "Transient: {} samples, duration {:.3} ms",
        trace.values.len(),
        trace.duration() * 1e3
    ));
    if ui.button("Open graph").clicked() {
        *open = true;
    }
}
