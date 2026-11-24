use super::*;
use crate::canvas::ZOOM_PRESETS;
use crate::icons::IconKind;
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
        self.analog_paused = false;
        let request = if self.transient_settings.adaptive {
            worker.request_transient(self.document.clone(), self.transient_settings)
        } else {
            self.transient = None;
            worker.start_transient(self.document.clone(), self.transient_settings)
        };
        self.analog_request = Some(request);
        self.analog_busy = true;
        self.message = "Transient running".to_string();
    }

    pub(super) fn pause_transient(&mut self) {
        if !self.analog_busy || self.analog_paused {
            return;
        }
        if self.simulation_domain == SimulationDomain::Mixed {
            if let Some(worker) = self.mixed_worker.as_ref() {
                worker.pause();
            }
            return;
        }
        if let Some(worker) = self.analog.as_ref() {
            worker.pause();
        }
    }

    pub(super) fn step_transient(&mut self) {
        if !self.analog_busy || !self.analog_paused {
            return;
        }
        if self.simulation_domain == SimulationDomain::Mixed {
            if let Some(worker) = self.mixed_worker.as_ref() {
                worker.step();
            }
            return;
        }
        if let Some(worker) = self.analog.as_ref() {
            worker.step();
        }
    }

    pub(super) fn reset_transient_session(&mut self) {
        if self.simulation_domain == SimulationDomain::Mixed {
            if let Some(worker) = self.mixed_worker.as_ref() {
                worker.reset();
            }
            self.analog_paused = true;
            return;
        }
        if let Some(worker) = self.analog.as_ref() {
            worker.reset();
        }
        self.analog_paused = true;
    }

    pub(super) fn run_analysis(&mut self) {
        if self.simulation_domain == SimulationDomain::Mixed {
            self.run_mixed();
            return;
        }
        if self.simulation_domain != SimulationDomain::Analog {
            self.message = "AVR runs from the board controls".to_string();
            return;
        }
        match self.analysis_kind {
            AnalysisKind::Dc => self.solve_dc(),
            AnalysisKind::Transient => self.run_transient(),
        }
    }

    pub(super) fn run_mixed(&mut self) {
        if self.mixed_worker.is_some() {
            if let Err(error) = self.start_mixed_worker() {
                self.message = error;
            }
            return;
        }
        match self.mixed_message() {
            Ok(message) => self.message = message,
            Err(error) => self.message = error,
        }
    }

    fn start_mixed_worker(&mut self) -> Result<(), String> {
        if self.firmware_words.is_empty() {
            return Err("Load firmware before mixed simulation".to_string());
        }
        let plan = {
            let component = self.mixed_component()?;
            self.mixed_plan(component)?
        };
        let Some(worker) = self.mixed_worker.as_mut() else {
            return Err("Mixed worker unavailable".to_string());
        };
        let request = worker.start(
            self.document.clone(),
            self.transient_settings,
            self.firmware_words.clone(),
            plan,
        );
        self.mixed_trace = None;
        self.mixed_request = Some(request);
        self.analog_busy = true;
        self.analog_paused = false;
        self.message = "Mixed running".to_string();
        Ok(())
    }

    fn mixed_message(&self) -> Result<String, String> {
        if self.firmware_words.is_empty() {
            return Err("Load firmware before mixed simulation".to_string());
        }
        let netlist = self.mixed_netlist()?;
        let component = self.mixed_component()?;
        let plan = self.mixed_plan(component)?;
        let config = Self::mixed_config(&self.transient_settings);
        let mut session = self.mixed_session(&netlist, &plan, &config)?;
        Self::run_mixed_steps(&mut session, config.steps)?;
        Ok(Self::format_mixed(&plan, &session))
    }

    fn mixed_netlist(&self) -> Result<crate::netlist::Netlist, String> {
        crate::netlist::extract(&self.document).map_err(|error| error.to_string())
    }

    pub(crate) fn mixed_net_names(&self) -> Vec<String> {
        crate::netlist::extract(&self.document).map_or_else(
            |_| Vec::new(),
            |netlist| netlist.nodes.iter().map(|node| node.name.clone()).collect(),
        )
    }

    fn mixed_component(&self) -> Result<&ComponentInstance, String> {
        self.document
            .components()
            .iter()
            .find(|component| component.definition == "atmega328p")
            .ok_or_else(|| "Schematic has no ATmega328P".to_string())
    }

    fn mixed_session(
        &self,
        netlist: &crate::netlist::Netlist,
        plan: &crate::netlist::MixedPinPlan,
        config: &analysis::TransientConfig,
    ) -> Result<mixed::AvrSession, String> {
        crate::netlist::build_mixed_session(netlist, self.firmware_words.clone(), plan, config)
    }

    fn run_mixed_steps(session: &mut mixed::AvrSession, steps: usize) -> Result<(), String> {
        for _ in 0..steps {
            if session.step().map_err(|error| error.to_string())?.is_none() {
                break;
            }
        }
        Ok(())
    }

    fn mixed_config(settings: &crate::transient::TransientSettings) -> analysis::TransientConfig {
        analysis::TransientConfig {
            dt: settings.dt,
            steps: settings.steps,
            gmin: settings.gmin,
            use_dc_op: settings.use_dc_op,
            ..Default::default()
        }
    }

    fn mixed_plan(
        &self,
        component: &ComponentInstance,
    ) -> Result<crate::netlist::MixedPinPlan, String> {
        let mut plan = self.empty_plan();
        for (&pin, &mode) in &component.pin_modes {
            Self::apply_pin_mode(&mut plan, pin, mode)?;
        }
        if Self::plan_is_empty(&plan) {
            return Err("Set at least one MCU pin mode".to_string());
        }
        Ok(plan)
    }

    fn empty_plan(&self) -> crate::netlist::MixedPinPlan {
        crate::netlist::MixedPinPlan {
            outputs: Vec::new(),
            adcs: Vec::new(),
            pwms: Vec::new(),
            uart_tx: None,
            uart_rx: None,
            high: self.mixed.high,
            low: self.mixed.low,
            external_resistance: self.mixed.resistance,
            adc_reference: self.mixed.adc_reference,
            uart_threshold: self.mixed.uart_threshold,
            probes: self.mixed_probes.clone(),
        }
    }

    fn plan_is_empty(plan: &crate::netlist::MixedPinPlan) -> bool {
        plan.outputs.is_empty()
            && plan.adcs.is_empty()
            && plan.pwms.is_empty()
            && plan.uart_tx.is_none()
            && plan.uart_rx.is_none()
    }

    fn apply_pin_mode(
        plan: &mut crate::netlist::MixedPinPlan,
        pin: u8,
        mode: McuPinMode,
    ) -> Result<(), String> {
        match mode {
            McuPinMode::Input => Ok(()),
            McuPinMode::Output => Self::push_output(plan, pin),
            McuPinMode::Adc => Self::push_adc(plan, pin),
            McuPinMode::Pwm => Self::push_pwm(plan, pin),
            McuPinMode::UartTx => Self::set_once(&mut plan.uart_tx, pin, "UART TX"),
            McuPinMode::UartRx => Self::set_once(&mut plan.uart_rx, pin, "UART RX"),
        }
    }

    fn push_output(plan: &mut crate::netlist::MixedPinPlan, pin: u8) -> Result<(), String> {
        plan.outputs.push(pin);
        Ok(())
    }

    fn push_adc(plan: &mut crate::netlist::MixedPinPlan, pin: u8) -> Result<(), String> {
        plan.adcs.push((Self::adc_channel(pin)?, pin));
        Ok(())
    }

    fn push_pwm(plan: &mut crate::netlist::MixedPinPlan, pin: u8) -> Result<(), String> {
        plan.pwms.push(pin);
        Ok(())
    }

    fn adc_channel(pin: u8) -> Result<u8, String> {
        match avr::AvrPackageMap::atmega328p_tqfp32().get(pin) {
            Some(avr::AvrSignal::Io { port: 1, bit }) => Ok(bit),
            _ => Err(format!("MCU pin {pin} is not an ADC pin")),
        }
    }

    fn set_once(slot: &mut Option<u8>, pin: u8, label: &str) -> Result<(), String> {
        if slot.is_some() {
            return Err(format!("Only one {label} pin is supported"));
        }
        *slot = Some(pin);
        Ok(())
    }

    fn format_mixed(plan: &crate::netlist::MixedPinPlan, session: &mixed::AvrSession) -> String {
        let values: Vec<String> = plan
            .outputs
            .iter()
            .enumerate()
            .map(|(index, pin)| {
                let voltage = session.node_voltage(index).unwrap_or(f64::NAN);
                format!("{pin}={voltage:.2}V")
            })
            .collect();
        format!("Mixed: {}", values.join(", "))
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
        while self.analog_busy {
            self.wait_one_analog();
        }
    }

    #[cfg(test)]
    pub(super) fn wait_one_analog(&mut self) {
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

    pub(super) fn poll_mixed(&mut self) {
        let Some(worker) = self.mixed_worker.as_ref() else {
            return;
        };
        for event in worker.poll() {
            self.apply_mixed_event(event);
        }
    }

    fn apply_mixed_event(&mut self, event: crate::mixed_worker::Event) {
        match event {
            crate::mixed_worker::Event::Started {
                request,
                outputs,
                adc_channels,
                pwm_pins,
                probes,
                steps,
            } => {
                let trace = crate::mixed_worker::MixedTrace {
                    outputs,
                    adc_channels,
                    pwm_pins,
                    probes,
                    samples: Vec::new(),
                };
                self.mixed_started(request, trace, steps);
            }
            crate::mixed_worker::Event::Samples { request, samples } => {
                self.mixed_samples(request, samples);
            }
            crate::mixed_worker::Event::Paused { request } => self.mixed_paused(request),
            crate::mixed_worker::Event::Finished { request } => self.mixed_finished(request),
            crate::mixed_worker::Event::Reset {
                request,
                outputs,
                adc_channels,
                pwm_pins,
                probes,
                steps,
            } => {
                let trace = crate::mixed_worker::MixedTrace {
                    outputs,
                    adc_channels,
                    pwm_pins,
                    probes,
                    samples: Vec::new(),
                };
                self.mixed_reset(request, trace, steps);
            }
            crate::mixed_worker::Event::Error { request, error } => {
                self.mixed_error(request, error);
            }
        }
    }

    fn mixed_started(
        &mut self,
        request: u64,
        trace: crate::mixed_worker::MixedTrace,
        steps: usize,
    ) {
        if self.mixed_request != Some(request) {
            return;
        }
        self.mixed_trace = Some(trace);
        self.analog_busy = true;
        self.analog_paused = false;
        self.message = format!("Mixed running: {steps} steps");
    }

    fn mixed_samples(&mut self, request: u64, samples: Vec<crate::mixed_worker::MixedSample>) {
        if self.mixed_request != Some(request) {
            return;
        }
        let Some(trace) = self.mixed_trace.as_mut() else {
            return;
        };
        trace.samples.extend(samples);
        if trace.samples.len() > 100_000 {
            let excess = trace.samples.len() - 100_000;
            trace.samples.drain(..excess);
        }
    }

    fn mixed_paused(&mut self, request: u64) {
        if self.mixed_request != Some(request) {
            return;
        }
        self.analog_busy = true;
        self.analog_paused = true;
        self.message = "Mixed paused".to_string();
    }

    fn mixed_finished(&mut self, request: u64) {
        if self.mixed_request != Some(request) {
            return;
        }
        self.analog_busy = false;
        self.analog_paused = false;
        let samples = self
            .mixed_trace
            .as_ref()
            .map_or(0, |trace| trace.samples.len());
        self.message = format!("Mixed finished: {samples} samples");
    }

    fn mixed_reset(&mut self, request: u64, trace: crate::mixed_worker::MixedTrace, steps: usize) {
        if self.mixed_request != Some(request) {
            return;
        }
        self.mixed_trace = Some(trace);
        self.message = format!("Mixed reset: {steps} steps");
    }

    fn mixed_error(&mut self, request: u64, error: String) {
        if self.mixed_request != Some(request) {
            return;
        }
        self.analog_busy = false;
        self.analog_paused = false;
        self.message = error;
    }

    fn apply_analog_event(&mut self, event: crate::analog::Event) {
        match event {
            crate::analog::Event::Dc { request, result } => self.apply_dc(request, result),
            crate::analog::Event::Transient { request, result } => {
                self.apply_transient(request, result);
            }
            crate::analog::Event::TransientStarted { request, metadata } => {
                self.apply_transient_started(request, metadata);
            }
            crate::analog::Event::TransientSamples { request, samples } => {
                self.apply_transient_samples(request, &samples);
            }
            crate::analog::Event::TransientPaused { request } => {
                self.apply_transient_paused(request);
            }
            crate::analog::Event::TransientFinished { request } => {
                self.apply_transient_finished(request);
            }
            crate::analog::Event::TransientReset { request } => {
                self.apply_transient_reset(request);
            }
            crate::analog::Event::TransientError { request, error } => {
                self.apply_transient_error(request, error);
            }
        }
    }

    fn apply_transient_started(&mut self, request: u64, metadata: crate::transient::TraceMetadata) {
        if Some(request) != self.analog_request {
            return;
        }
        let trace = TransientTrace::empty(metadata);
        self.transient_selected = default_transient_selection(&trace);
        self.transient_selected_currents.clear();
        self.transient = Some(trace);
        self.transient_open = true;
        self.analog_busy = true;
        self.analog_paused = false;
        self.message = "Transient running".to_string();
    }

    fn apply_transient_samples(&mut self, request: u64, samples: &[analysis::TransientSample]) {
        if Some(request) != self.analog_request {
            return;
        }
        let Some(trace) = self.transient.as_mut() else {
            return;
        };
        for sample in samples {
            trace.push(sample);
        }
        self.message = format!("Transient: {} samples", trace.values.len());
    }

    fn apply_transient_paused(&mut self, request: u64) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_paused = true;
        self.message = "Transient paused".to_string();
    }

    fn apply_transient_finished(&mut self, request: u64) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_busy = false;
        self.analog_paused = false;
        self.message = "Transient complete".to_string();
    }

    fn apply_transient_reset(&mut self, request: u64) {
        if Some(request) != self.analog_request {
            return;
        }
        if let Some(trace) = self.transient.as_mut() {
            trace.clear_samples();
        }
        self.analog_paused = true;
        self.message = "Transient reset".to_string();
    }

    fn apply_transient_error(&mut self, request: u64, error: String) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_busy = false;
        self.analog_paused = false;
        self.message = error;
    }

    fn apply_dc(&mut self, request: u64, result: Result<crate::analog::DcResultView, String>) {
        if Some(request) != self.analog_request {
            return;
        }
        self.analog_busy = false;
        self.analog_paused = false;
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
        for tool in Tool::ALL {
            let button = icon_button(tool.icon(), tool.label()).selected(self.tool == tool);
            let button = match tool.shortcut() {
                Some(shortcut) => button.shortcut(shortcut),
                None => button,
            };
            if ui.add(button).clicked() {
                self.set_tool(tool);
            }
        }
    }

    pub(super) fn sheet_buttons(&mut self, ui: &mut egui::Ui) {
        panel_button(ui, IconKind::Folder, "Library", &mut self.library_open, "L");
        panel_button(
            ui,
            IconKind::Settings,
            "Inspector",
            &mut self.inspector_open,
            "I",
        );
        panel_button(ui, IconKind::Palette, "Legend", &mut self.legend_open, "K");
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
        self.grid_menu(ui);
        self.units_control(ui);
        self.zoom_controls(ui);
        toggle_button(ui, IconKind::Map, "Mini", &mut self.minimap_open);
        toggle_button(ui, IconKind::Route, "Flow", &mut self.flow_animation);
        if self.flow_animation {
            ui.ctx().request_repaint();
        }
        self.compact_toggle(ui);
        self.theme_button(ui);
    }

    fn compact_toggle(&mut self, ui: &mut egui::Ui) {
        let button = icon_button(IconKind::Settings, "Icons")
            .selected(self.compact_icons)
            .tooltip("Icon-only toolbar");
        if ui.add(button).clicked() {
            self.compact_icons = !self.compact_icons;
        }
    }

    fn theme_button(&mut self, ui: &mut egui::Ui) {
        let (icon, label) = if self.light_theme {
            (IconKind::Moon, "Dark")
        } else {
            (IconKind::Sun, "Light")
        };
        let button = icon_button(icon, label).selected(self.light_theme);
        if ui.add(button).clicked() {
            self.light_theme = !self.light_theme;
            ui.ctx().set_theme(theme_preference(self.light_theme));
        }
    }

    fn grid_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(format!("Grid x{}", self.grid_step.label()), |ui| {
            icon_heading(ui, IconKind::Grid, "Grid");
            ui.checkbox(&mut self.grid_visible, "Show grid");
            ui.separator();
            self.grid_style_choices(ui);
            ui.separator();
            self.grid_step_choices(ui);
            ui.checkbox(&mut self.grid_axes, "Axes");
            ui.checkbox(&mut self.grid_rulers, "Rulers");
            ui.separator();
            self.snap_choices(ui);
            ui.separator();
            if ui.button("Settings...").clicked() {
                self.grid_settings_open = true;
                ui.close();
            }
        });
    }

    fn grid_style_choices(&mut self, ui: &mut egui::Ui) {
        for style in canvas::GridStyle::ALL {
            ui.selectable_value(&mut self.grid_style, style, style.label());
        }
    }

    fn grid_step_choices(&mut self, ui: &mut egui::Ui) {
        for step in GridStep::ALL {
            ui.selectable_value(&mut self.grid_step, step, format!("Step {}", step.label()));
        }
    }

    fn snap_choices(&mut self, ui: &mut egui::Ui) {
        for mode in SnapMode::ALL {
            ui.selectable_value(&mut self.snap_mode, mode, mode.label());
        }
        ui.checkbox(&mut self.snap_objects, "Snap objects");
    }

    fn units_control(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            icon_slot(ui, IconKind::Ruler);
            egui::ComboBox::from_id_salt("units")
                .selected_text(self.units.label())
                .show_ui(ui, |ui| {
                    for units in Units::ALL {
                        ui.selectable_value(&mut self.units, units, units.label());
                    }
                });
        });
    }

    pub(super) fn zoom_controls(&mut self, ui: &mut egui::Ui) {
        let out = icon_button(IconKind::ZoomOut, "")
            .tooltip("Zoom out")
            .shortcut("F2");
        if ui.add(out).clicked() {
            self.zoom_to(self.camera.zoom / 1.3);
        }
        self.zoom_preset_combo(ui);
        let zoom_in = icon_button(IconKind::ZoomIn, "")
            .tooltip("Zoom in")
            .shortcut("F1");
        if ui.add(zoom_in).clicked() {
            self.zoom_to(self.camera.zoom * 1.3);
        }
        self.fit_menu(ui);
    }

    fn zoom_preset_combo(&mut self, ui: &mut egui::Ui) {
        let current = self.camera.zoom;
        let mut chosen = current;
        egui::ComboBox::from_id_salt("zoom_preset")
            .selected_text(format!("{:.0}%", current * 100.0))
            .show_ui(ui, |ui| {
                for &zoom in ZOOM_PRESETS {
                    ui.selectable_value(&mut chosen, zoom, format!("{:.0}%", zoom * 100.0));
                }
            });
        if chosen != current {
            self.zoom_to(chosen);
        }
    }

    fn fit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Fit", |ui| {
            for (label, target, icon) in [
                ("All", FitTarget::All, IconKind::Fit),
                ("Objects", FitTarget::Objects, IconKind::Package),
                ("Selection", FitTarget::Selection, IconKind::Pointer),
            ] {
                if ui.add(icon_button(icon, label)).clicked() {
                    self.fit_view(target);
                    ui.close();
                }
            }
        });
    }

    pub(super) fn zoom_to(&mut self, zoom: f64) {
        let target = GridCamera::clamp_zoom(zoom);
        if (target - self.camera.zoom).abs() < f64::EPSILON {
            return;
        }
        let Some(rect) = self.canvas_rect else {
            self.camera.zoom = target;
            return;
        };
        let factor = target / self.camera.zoom;
        self.camera.zoom_at(rect, rect.center(), factor);
    }

    pub(super) fn fit_view(&mut self, target: FitTarget) {
        let Some(rect) = self.canvas_rect else {
            return;
        };
        let selection = match target {
            FitTarget::Selection => Some(&self.selection),
            FitTarget::All | FitTarget::Objects => None,
        };
        let Some((min, max)) = content_bounds(&self.document, self.library, selection) else {
            return;
        };
        let margin = fit_margin(rect);
        self.camera
            .fit_world(rect, grid_pos_f(min), grid_pos_f(max), margin);
    }

    pub(super) fn wire_controls(&mut self, ui: &mut egui::Ui) {
        icon_heading(ui, IconKind::Route, "Wire");
        self.wire_class_control(ui);
        self.route_buttons(ui);
        let finish = icon_button(IconKind::Check, "Finish").shortcut("Enter");
        if ui.add(finish).clicked() {
            self.finish_wire();
        }
        let cancel = icon_button(IconKind::Close, "Cancel").shortcut("Esc");
        if ui.add(cancel).clicked() {
            self.wire_points.clear();
        }
    }

    fn wire_class_control(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            icon_slot(ui, IconKind::Junction);
            egui::ComboBox::from_id_salt("wire_class")
                .selected_text(self.wire_class.label())
                .show_ui(ui, |ui| {
                    for class in NetClass::ALL {
                        ui.selectable_value(&mut self.wire_class, class, class.label());
                    }
                });
        });
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
        let target = self.snap_world(self.pointer_world?).point;
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

pub(crate) fn fit_margin(rect: egui::Rect) -> f64 {
    if rect.height() < 768.0 { 1.10 } else { 1.04 }
}

fn toggle_button(ui: &mut egui::Ui, icon: IconKind, label: &'static str, value: &mut bool) {
    let button = icon_button(icon, label).selected(*value);
    if ui.add(button).clicked() {
        *value = !*value;
    }
}

/// Returns the egui theme preference for the light-theme flag.
pub(crate) fn theme_preference(light: bool) -> egui::ThemePreference {
    if light {
        egui::ThemePreference::Light
    } else {
        egui::ThemePreference::Dark
    }
}

fn panel_button(
    ui: &mut egui::Ui,
    icon: IconKind,
    label: &'static str,
    open: &mut bool,
    shortcut: &'static str,
) {
    let button = icon_button(icon, label).selected(*open).shortcut(shortcut);
    if ui.add(button).clicked() {
        *open = true;
    }
}

pub(super) fn default_transient_selection(trace: &TransientTrace) -> BTreeSet<usize> {
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
