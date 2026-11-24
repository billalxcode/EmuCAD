use super::*;

fn test_input() -> eframe::egui::RawInput {
    eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}

#[test]
fn tool_shortcuts_switch_modes() {
    for (key, tool) in [
        (eframe::egui::Key::V, Tool::Select),
        (eframe::egui::Key::P, Tool::Place),
        (eframe::egui::Key::W, Tool::Wire),
        (eframe::egui::Key::G, Tool::Probe),
    ] {
        let input = eframe::egui::RawInput {
            events: vec![key_event(key)],
            ..test_input()
        };
        let mut app = GuiApp::default();
        let mut output = eframe::egui::Context::default().run_ui(input, |ui| app.handle_keys(ui));
        output.textures_delta.clear();
        assert_eq!(app.tool, tool);
    }
}

#[test]
fn panel_shortcut_toggles_library() {
    let input = eframe::egui::RawInput {
        events: vec![key_event(eframe::egui::Key::L)],
        ..test_input()
    };
    let mut app = GuiApp::default();
    let mut output = eframe::egui::Context::default().run_ui(input, |ui| app.handle_keys(ui));
    output.textures_delta.clear();
    assert!(app.library_open);
}

#[test]
fn toolbar_renders_with_icons() {
    let mut app = GuiApp::default();
    let mut output = eframe::egui::Context::default().run_ui(test_input(), |ui| app.toolbar(ui));
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn palette_renders_with_icons() {
    let mut app = GuiApp::default();
    app.palette.open = true;
    let mut output =
        eframe::egui::Context::default().run_ui(test_input(), |ui| app.command_palette(ui));
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn tool_icons_are_distinct() {
    let icons: std::collections::BTreeSet<_> = Tool::ALL.iter().map(|tool| tool.icon()).collect();
    assert_eq!(icons.len(), Tool::ALL.len());
}
