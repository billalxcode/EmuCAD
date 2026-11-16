use super::*;
use crate::transient::TransientSettings;

fn worker_app() -> GuiApp {
    GuiApp {
        document: crate::analog::tests::test_document(),
        analog: Some(crate::analog::AnalogHandle::spawn()),
        transient_settings: TransientSettings {
            dt: 1e-6,
            steps: 2,
            use_dc_op: true,
            ..TransientSettings::default()
        },
        ..GuiApp::default()
    }
}

#[test]
fn transient_worker_installs_trace() {
    let mut app = worker_app();
    app.run_transient();
    app.wait_analog();
    assert!(app.transient.is_some());
    assert!(app.message.contains("Transient"));
}

#[test]
fn dc_worker_installs_result() {
    let mut app = worker_app();
    app.solve_dc();
    app.wait_analog();
    assert!(app.analog_dc.is_some());
}

#[test]
fn simulation_window_renders_dc_results() {
    let mut app = GuiApp {
        document: crate::analog::tests::test_document(),
        simulation_open: true,
        analog_dc: Some(crate::analog::DcResultView {
            voltages: vec![("0".to_string(), 0.0)],
            iterations: 3,
            residual: 1e-12,
        }),
        ..GuiApp::default()
    };
    let output = render_simulation(&mut app);
    assert!(!output.shapes.is_empty());
}

fn render_simulation(app: &mut GuiApp) -> eframe::egui::FullOutput {
    let context = eframe::egui::Context::default();
    let input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(640.0, 360.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| app.simulation_window(ui));
    output.textures_delta.clear();
    output
}
