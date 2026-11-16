use super::*;

#[test]
fn analog_plot_maps_cycle_and_value() {
    let plot = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(10.0, 20.0),
        eframe::egui::vec2(100.0, 50.0),
    );
    assert!((super::cycle_to_x(plot, (0, 100), 50) - 60.0).abs() < 0.01);
    assert_eq!(super::cycle_at(plot, (0, 100), 60.0), 50);
    assert!((super::value_to_y(plot, (0.0, 10.0), 10.0) - 20.0).abs() < 0.01);
}

#[test]
fn analog_graph_renders_in_egui_context() {
    let (probes, mut view) = analog_fixture();
    let mut output = render_analog(&probes, &mut view);
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

fn analog_fixture() -> (crate::probe::ProbeSet, crate::probe::PlotView) {
    let mut probes = crate::probe::ProbeSet::default();
    assert!(probes.add(analog_spec()));
    probes.sample_from(&analog_provider(), 100);
    (probes, crate::probe::PlotView::default())
}

fn analog_spec() -> crate::probe::ProbeSpec {
    crate::probe::ProbeSpec {
        label: "PB5".to_string(),
        unit: "V",
        color: eframe::egui::Color32::WHITE,
        source: crate::probe::ProbeSource::Pin {
            component: ComponentId(0),
            pin: "PB5",
        },
        sampling: crate::probe::Sampling::default(),
    }
}

fn analog_provider() -> crate::probe::SampleProvider {
    crate::probe::SampleProvider {
        sample: crate::sim::Sample {
            cycle: 100,
            pb5_high: true,
            pb5_millivolts: 5000,
            led_on: true,
            led_microamps: 14_000,
        },
    }
}

fn render_analog(
    probes: &crate::probe::ProbeSet,
    view: &mut crate::probe::PlotView,
) -> eframe::egui::FullOutput {
    eframe::egui::Context::default().run_ui(eframe::egui::RawInput::default(), |ui| {
        super::show_analog_graph(ui, probes, view, crate::sim::Status::default());
    })
}

#[test]
fn flow_phase_is_optional() {
    assert_eq!(super::flow_phase(false, 1.0), None);
    let phase = super::flow_phase(true, 1.0).expect("phase");
    assert!((0.0..1000.0).contains(&phase));
}

#[test]
fn transient_graph_renders_in_egui_context() {
    let trace = crate::transient::TransientTrace {
        times: vec![0.0, 1e-3, 2e-3],
        nodes: vec!["0".to_string(), "out".to_string()],
        values: vec![vec![0.0, 0.0], vec![0.0, 2.5], vec![0.0, 5.0]],
        devices: Vec::new(),
        currents: vec![Vec::new(); 3],
        dt: 1e-3,
    };
    let mut selected = std::collections::BTreeSet::from([1]);
    let mut selected_currents = std::collections::BTreeSet::new();
    let context = eframe::egui::Context::default();
    let input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(640.0, 360.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        crate::app::show_transient(ui, &trace, &mut selected, &mut selected_currents);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}
