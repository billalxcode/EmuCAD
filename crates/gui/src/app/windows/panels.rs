use super::*;
use crate::icons::IconKind;
use crate::mixed_worker::{MixedSample, MixedTrace};

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
        let mixed_nets = self.mixed_net_names();
        let mut mixed = self.mixed;
        let mut mixed_probes = self.mixed_probes.clone();
        let mut mixed_settings = self.transient_settings;
        let domain = &mut self.simulation_domain;
        let previous = self.analysis_kind;
        let kind = &mut self.analysis_kind;
        let dc = self.analog_dc.as_ref();
        let trace = self.transient.as_ref();
        let mixed_trace = self.mixed_trace.as_ref();
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
                    if ui
                        .add_enabled(!busy, icon_button(IconKind::Run, "Run"))
                        .clicked()
                    {
                        run = true;
                    }
                    if ui
                        .add_enabled(busy && !paused, icon_button(IconKind::Pause, "Pause"))
                        .clicked()
                    {
                        pause = true;
                    }
                    if ui
                        .add_enabled(busy && paused, icon_button(IconKind::Step, "Step"))
                        .clicked()
                    {
                        step = true;
                    }
                    if ui
                        .add_enabled(busy || has_trace, icon_button(IconKind::Reset, "Reset"))
                        .clicked()
                    {
                        reset = true;
                    }
                });
                if *domain == crate::analysis::SimulationDomain::Mixed {
                    show_mixed_controls(
                        ui,
                        &mixed_nets,
                        &mut mixed,
                        &mut mixed_probes,
                        &mut mixed_settings,
                    );
                    show_mixed_summary(ui, mixed_trace);
                } else if *kind == crate::analysis::AnalysisKind::Dc {
                    show_dc_results(ui, dc);
                } else {
                    show_transient_summary(ui, trace, &mut open_transient);
                }
            });
        self.simulation_open = open;
        self.mixed = mixed;
        self.mixed_probes = mixed_probes;
        if self.transient_settings != mixed_settings {
            self.transient_settings = mixed_settings;
        }
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

fn show_mixed_controls(
    ui: &mut egui::Ui,
    nets: &[String],
    mixed: &mut MixedSettings,
    probes: &mut Vec<String>,
    settings: &mut crate::transient::TransientSettings,
) {
    icon_heading(ui, IconKind::Settings, "Mixed settings");
    show_mixed_levels(ui, mixed);
    show_mixed_timing(ui, settings);
    show_mixed_probes(ui, nets, probes);
}

fn show_mixed_levels(ui: &mut egui::Ui, mixed: &mut MixedSettings) {
    egui::Grid::new("mixed_levels")
        .num_columns(3)
        .show(ui, |ui| {
            mixed_number(ui, "High", &mut mixed.high, "V");
            mixed_number(ui, "Low", &mut mixed.low, "V");
            mixed_number(ui, "Resistance", &mut mixed.resistance, "ohm");
            mixed_number(ui, "ADC ref", &mut mixed.adc_reference, "V");
            mixed_number(ui, "UART threshold", &mut mixed.uart_threshold, "V");
        });
}

fn mixed_number(ui: &mut egui::Ui, label: &str, value: &mut f64, unit: &str) {
    ui.label(label);
    ui.add(egui::DragValue::new(value).speed(0.1));
    ui.label(unit);
    ui.end_row();
}

fn show_mixed_timing(ui: &mut egui::Ui, settings: &mut crate::transient::TransientSettings) {
    ui.horizontal(|ui| {
        ui.label("dt");
        ui.add(egui::DragValue::new(&mut settings.dt).speed(1e-6));
        ui.label("s");
        ui.label("steps");
        ui.add(egui::DragValue::new(&mut settings.steps));
    });
    let duration = settings.dt * settings.steps as f64 * 1e3;
    ui.label(format!("Duration {duration:.3} ms"));
}

fn show_mixed_probes(ui: &mut egui::Ui, nets: &[String], probes: &mut Vec<String>) {
    icon_heading(ui, IconKind::Crosshair, "Net probes");
    let mut chosen = None;
    egui::ComboBox::from_id_salt("mixed_probe_net")
        .selected_text("Add net")
        .show_ui(ui, |ui| {
            for name in nets {
                if ui.selectable_label(false, name).clicked() {
                    chosen = Some(name.clone());
                }
            }
        });
    if let Some(name) = chosen {
        if !probes.contains(&name) {
            probes.push(name);
        }
    }
    show_mixed_probe_list(ui, probes);
}

fn show_mixed_probe_list(ui: &mut egui::Ui, probes: &mut Vec<String>) {
    let mut remove = None;
    ui.horizontal_wrapped(|ui| {
        for name in probes.iter() {
            let button = icon_button(IconKind::Close, name.clone()).tooltip("Remove probe");
            if ui.add(button).clicked() {
                remove = Some(name.clone());
            }
        }
    });
    if let Some(name) = remove {
        probes.retain(|probe| *probe != name);
    }
}

fn show_mixed_summary(ui: &mut egui::Ui, trace: Option<&crate::mixed_worker::MixedTrace>) {
    let Some(trace) = trace else {
        ui.label("No mixed result");
        return;
    };
    icon_heading(ui, IconKind::Chart, "Mixed trace");
    ui.label(format!("{} samples", trace.samples.len()));
    let Some(last) = trace.samples.last() else {
        return;
    };
    show_mixed_values(ui, trace, last);
    show_mixed_graph(ui, trace);
}

fn show_mixed_values(ui: &mut egui::Ui, trace: &MixedTrace, sample: &MixedSample) {
    egui::Grid::new("mixed_values")
        .num_columns(2)
        .show(ui, |ui| {
            for (index, label) in mixed_series_labels(trace).iter().enumerate() {
                ui.label(label);
                ui.label(format!("{:.2} V", mixed_series_value(trace, sample, index)));
                ui.end_row();
            }
        });
}

fn show_mixed_graph(ui: &mut egui::Ui, trace: &MixedTrace) {
    let count = mixed_series_labels(trace).len();
    if trace.samples.len() < 2 || count == 0 {
        return;
    }
    let height = 120.0;
    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
    for series in 0..count {
        let color = mixed_series_color(series);
        let points = mixed_series_points(rect, trace, series);
        painter.add(egui::Shape::line(points, egui::Stroke::new(1.0, color)));
    }
    show_mixed_legend(ui, trace);
}

fn show_mixed_legend(ui: &mut egui::Ui, trace: &MixedTrace) {
    ui.horizontal_wrapped(|ui| {
        for (index, label) in mixed_series_labels(trace).iter().enumerate() {
            ui.colored_label(mixed_series_color(index), label);
        }
    });
}

fn mixed_series_labels(trace: &MixedTrace) -> Vec<String> {
    let mut labels: Vec<String> = trace
        .outputs
        .iter()
        .map(|pin| format!("Pin {pin}"))
        .collect();
    labels.extend(
        trace
            .adc_channels
            .iter()
            .map(|channel| format!("ADC {channel}")),
    );
    labels.extend(trace.pwm_pins.iter().map(|pin| format!("PWM {pin}")));
    labels.extend(trace.probes.iter().cloned());
    labels
}

fn mixed_series_value(trace: &MixedTrace, sample: &MixedSample, index: usize) -> f64 {
    let mut offset = index;
    if offset < trace.outputs.len() {
        return sample
            .outputs
            .get(offset)
            .map_or(f64::NAN, |(_, value)| *value);
    }
    offset -= trace.outputs.len();
    if offset < trace.adc_channels.len() {
        return sample.adcs.get(offset).copied().unwrap_or(f64::NAN);
    }
    offset -= trace.adc_channels.len();
    if offset < trace.pwm_pins.len() {
        return sample.pwms.get(offset).copied().unwrap_or(f64::NAN);
    }
    offset -= trace.pwm_pins.len();
    sample.probes.get(offset).copied().unwrap_or(f64::NAN)
}

fn mixed_series_points(rect: egui::Rect, trace: &MixedTrace, series: usize) -> Vec<egui::Pos2> {
    let last = (trace.samples.len() - 1) as f32;
    trace
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let voltage = mixed_series_value(trace, sample, series);
            let x = rect.left() + rect.width() * (index as f32 / last);
            let y = rect.bottom() - rect.height() * (voltage.clamp(0.0, 5.0) as f32 / 5.0);
            egui::pos2(x, y)
        })
        .collect()
}

fn mixed_series_color(series: usize) -> egui::Color32 {
    const COLORS: [egui::Color32; 6] = [
        egui::Color32::LIGHT_GREEN,
        egui::Color32::LIGHT_BLUE,
        egui::Color32::YELLOW,
        egui::Color32::ORANGE,
        egui::Color32::LIGHT_RED,
        egui::Color32::CYAN,
    ];
    COLORS[series % COLORS.len()]
}

#[cfg(test)]
mod tests {
    use super::{mixed_series_labels, mixed_series_value, show_mixed_controls, show_mixed_summary};
    use crate::mixed_worker::{MixedSample, MixedTrace};

    fn sample(output: f64, adc: f64, pwm: f64, probes: [f64; 2]) -> MixedSample {
        MixedSample {
            time: kernel::Time::ZERO,
            cycles: 0,
            outputs: vec![(17, output)],
            adcs: vec![adc],
            pwms: vec![pwm],
            probes: probes.to_vec(),
        }
    }

    fn trace() -> MixedTrace {
        MixedTrace {
            outputs: vec![17],
            adc_channels: vec![0],
            pwm_pins: vec![5],
            probes: vec!["VOUT".to_string(), "VIN".to_string()],
            samples: vec![sample(5.0, 2.5, 1.0, [3.0, 4.0])],
        }
    }

    #[test]
    fn mixed_series_cover_outputs_adc_pwm_and_probes() {
        let trace = trace();
        assert_eq!(
            mixed_series_labels(&trace),
            vec!["Pin 17", "ADC 0", "PWM 5", "VOUT", "VIN"]
        );
        let sample = &trace.samples[0];
        assert_eq!(mixed_series_value(&trace, sample, 0), 5.0);
        assert_eq!(mixed_series_value(&trace, sample, 1), 2.5);
        assert_eq!(mixed_series_value(&trace, sample, 2), 1.0);
        assert_eq!(mixed_series_value(&trace, sample, 3), 3.0);
        assert_eq!(mixed_series_value(&trace, sample, 4), 4.0);
    }

    #[test]
    fn mixed_summary_and_controls_render() {
        let trace = trace();
        let mut mixed = crate::app::MixedSettings::default();
        let mut probes = Vec::new();
        let mut settings = crate::transient::TransientSettings::default();
        let nets = vec!["0".to_string(), "VOUT".to_string()];
        let mut output = eframe::egui::Context::default().run_ui(Default::default(), |ui| {
            show_mixed_controls(ui, &nets, &mut mixed, &mut probes, &mut settings);
            show_mixed_summary(ui, Some(&trace));
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }
}
