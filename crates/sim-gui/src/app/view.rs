use super::*;
use crate::transient::TransientTrace;

impl GuiApp {
    pub(super) fn check_netlist(&mut self) {
        self.message = match crate::netlist::extract(&self.document) {
            Ok(netlist) => format!(
                "Netlist: {} nodes, {} parts",
                netlist.nodes.len(),
                netlist.components.len()
            ),
            Err(error) => error.to_string(),
        };
    }

    pub(super) fn solve_dc(&mut self) {
        if self.analog_busy {
            return;
        }
        let Some(worker) = self.analog.as_mut() else {
            self.message = "Analog worker unavailable".to_string();
            return;
        };
        self.analog_request = Some(worker.request_dc(self.document.clone()));
        self.analog_busy = true;
        self.message = "DC running".to_string();
    }

    pub(super) fn run_transient(&mut self) {
        if self.analog_busy {
            return;
        }
        let Some(worker) = self.analog.as_mut() else {
            self.message = "Analog worker unavailable".to_string();
            return;
        };
        let request = worker.request_transient(self.document.clone(), self.transient_settings);
        self.analog_request = Some(request);
        self.analog_busy = true;
        self.message = "Transient running".to_string();
    }

    pub(super) fn run_analysis(&mut self) {
        if self.simulation_domain != SimulationDomain::Analog {
            self.message = match self.simulation_domain {
                SimulationDomain::Avr => "AVR runs from the board controls".to_string(),
                SimulationDomain::Mixed => "Mixed simulation is not implemented".to_string(),
                SimulationDomain::Analog => String::new(),
            };
            return;
        }
        match self.analysis_kind {
            AnalysisKind::Dc => self.solve_dc(),
            AnalysisKind::Transient => self.run_transient(),
        }
    }

    pub(super) fn analysis_state(&self) -> AnalysisState {
        AnalysisState {
            domain: self.simulation_domain,
            kind: self.analysis_kind,
            transient: self.transient_settings,
        }
    }

    pub(super) fn apply_analysis_state(&mut self, state: AnalysisState) {
        self.simulation_domain = state.domain;
        self.analysis_kind = state.kind;
        self.transient_settings = state.transient;
    }

    #[cfg(test)]
    pub(super) fn wait_analog(&mut self) {
        let Some(worker) = self.analog.as_ref() else {
            return;
        };
        let event = worker.wait();
        self.apply_analog_event(event);
    }

    pub(super) fn poll_analog(&mut self) {
        let Some(worker) = self.analog.as_ref() else {
            return;
        };
        let events = worker.poll();
        for event in events {
            self.apply_analog_event(event);
        }
    }

    fn apply_analog_event(&mut self, event: crate::analog::Event) {
        match event {
            crate::analog::Event::Dc { request, result } => self.apply_dc(request, result),
            crate::analog::Event::Transient { request, result } => {
                self.apply_transient(request, result);
            }
        }
    }

    fn apply_dc(&mut self, request: u64, result: Result<crate::analog::DcResultView, String>) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_busy = false;
        match result {
            Ok(dc) => {
                self.message = format_dc(&dc.voltages);
                self.analog_dc = Some(dc);
            }
            Err(error) => self.message = error,
        }
    }

    fn apply_transient(&mut self, request: u64, result: Result<TransientTrace, String>) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_busy = false;
        match result {
            Ok(trace) => self.install_transient(trace),
            Err(error) => self.message = error,
        }
    }

    fn install_transient(&mut self, trace: TransientTrace) {
        self.transient_selected = default_transient_selection(&trace);
        self.transient_selected_currents.clear();
        self.message = format!(
            "Transient: {} samples, {:.3} ms",
            trace.values.len(),
            trace.duration() * 1e3
        );
        self.transient = Some(trace);
        self.transient_open = true;
    }

    pub(super) fn tool_buttons(&mut self, ui: &mut egui::Ui) {
        let previous = self.tool;
        for tool in Tool::ALL {
            let text = format!("{} {}", tool.icon(), tool.label());
            ui.selectable_value(&mut self.tool, tool, text);
        }
        if previous != self.tool {
            self.wire_points.clear();
        }
    }

    pub(super) fn sheet_buttons(&mut self, ui: &mut egui::Ui) {
        if ui
            .selectable_label(self.library_open, "[L] Library")
            .clicked()
        {
            self.library_open = true;
        }
        if ui
            .selectable_label(self.inspector_open, "[I] Inspector")
            .clicked()
        {
            self.inspector_open = true;
        }
        if ui
            .selectable_label(self.legend_open, "[K] Legend")
            .clicked()
        {
            self.legend_open = true;
        }
    }

    pub(super) fn toolbar_view(&mut self, ui: &mut egui::Ui) {
        self.view_options(ui);
        ui.separator();
        if self.tool == Tool::Wire {
            self.wire_controls(ui);
        } else {
            self.placement_controls(ui);
        }
    }

    pub(super) fn view_options(&mut self, ui: &mut egui::Ui) {
        self.zoom_controls(ui);
        ui.checkbox(&mut self.grid_visible, "Grid");
        ui.checkbox(&mut self.minimap_open, "Mini");
        ui.checkbox(&mut self.flow_animation, "Flow");
        if self.flow_animation {
            ui.ctx().request_repaint();
        }
        if ui.checkbox(&mut self.light_theme, "Light").changed() {
            ui.ctx().set_theme(if self.light_theme {
                egui::ThemePreference::Light
            } else {
                egui::ThemePreference::Dark
            });
        }
    }

    pub(super) fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("zoom_preset")
            .selected_text(format!("{:.0}%", self.camera.zoom * 100.0))
            .show_ui(ui, |ui| {
                for zoom in [0.5, 1.0, 2.0] {
                    ui.selectable_value(
                        &mut self.camera.zoom,
                        zoom,
                        format!("{:.0}%", zoom * 100.0),
                    );
                }
            });
        if ui.button("Fit").clicked() {
            self.fit_content(false);
        }
        if ui.button("Fit Sel").clicked() {
            self.fit_content(true);
        }
    }

    pub(super) fn fit_content(&mut self, selection_only: bool) {
        let Some(rect) = self.canvas_rect else {
            return;
        };
        let selection = selection_only.then_some(&self.selection);
        let Some((min, max)) = content_bounds(&self.document, self.library, selection) else {
            return;
        };
        self.camera
            .fit_world(rect, grid_pos_f(min), grid_pos_f(max), 24.0);
    }

    pub(super) fn wire_controls(&mut self, ui: &mut egui::Ui) {
        ui.label("Wire");
        ui.label("Class");
        egui::ComboBox::from_id_salt("wire_class")
            .selected_text(self.wire_class.label())
            .show_ui(ui, |ui| {
                for class in NetClass::ALL {
                    ui.selectable_value(&mut self.wire_class, class, class.label());
                }
            });
        self.route_buttons(ui);
        if ui.button("Finish").clicked() {
            self.finish_wire();
        }
        if ui.button("Cancel").clicked() {
            self.wire_points.clear();
        }
    }

    pub(super) fn route_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("L-H").clicked() {
            self.apply_l_route(true);
        }
        if ui.button("L-V").clicked() {
            self.apply_l_route(false);
        }
        if ui.button("Z-H").clicked() {
            self.apply_z_route(true);
        }
        if ui.button("Z-V").clicked() {
            self.apply_z_route(false);
        }
    }

    pub(super) fn apply_l_route(&mut self, horizontal_first: bool) {
        self.apply_route(l_route, horizontal_first);
    }

    pub(super) fn apply_z_route(&mut self, horizontal_first: bool) {
        self.apply_route(z_route, horizontal_first);
    }

    pub(super) fn apply_route(
        &mut self,
        build: fn(GridPos, GridPos, bool) -> Vec<GridPos>,
        horizontal_first: bool,
    ) {
        let Some(path) = self.route_path(build, horizontal_first) else {
            return;
        };
        self.wire_points.extend(path);
    }

    pub(super) fn route_path(
        &self,
        build: fn(GridPos, GridPos, bool) -> Vec<GridPos>,
        horizontal_first: bool,
    ) -> Option<Vec<GridPos>> {
        let last = *self.wire_points.last()?;
        let target = world_to_grid(self.pointer_world?);
        let path = build(last, target, horizontal_first);
        let mut current = last;
        for &point in &path {
            if !self.document.can_add_segment(self.library, current, point) {
                return None;
            }
            current = point;
        }
        Some(path)
    }

    pub(super) fn placement_controls(&mut self, ui: &mut egui::Ui) {
        if let Some(definition) = self.placing {
            ui.label(format!("Placing {}", definition.name));
            if ui.button("Rotate").clicked() {
                self.rotate();
            }
            if ui.button("Mirror").clicked() {
                self.mirror();
            }
            if ui.button("Cancel").clicked() {
                self.placing = None;
            }
        }
    }
}

fn default_transient_selection(trace: &TransientTrace) -> BTreeSet<usize> {
    trace
        .nodes
        .iter()
        .enumerate()
        .skip(1)
        .take(3)
        .map(|(index, _)| index)
        .collect()
}

fn format_dc(voltages: &[(String, f64)]) -> String {
    let mut text = String::from("DC:");
    for (name, value) in voltages.iter().take(4) {
        text.push_str(&format!(" {name}={value:.3} V"));
    }
    text
}
