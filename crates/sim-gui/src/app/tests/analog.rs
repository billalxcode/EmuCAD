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
