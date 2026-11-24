use super::*;

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

fn press_key(app: &mut GuiApp, key: eframe::egui::Key, modifiers: eframe::egui::Modifiers) {
    let input = eframe::egui::RawInput {
        events: vec![
            eframe::egui::Event::ModifiersChanged(modifiers),
            key_event_with(key, modifiers),
        ],
        ..test_input()
    };
    let mut output = eframe::egui::Context::default().run_ui(input, |ui| app.handle_keys(ui));
    output.textures_delta.clear();
}

#[test]
fn grid_shortcut_cycles_steps() {
    let mut app = GuiApp {
        canvas_focused: true,
        ..GuiApp::default()
    };
    press_key(
        &mut app,
        eframe::egui::Key::N,
        eframe::egui::Modifiers::NONE,
    );
    assert_eq!(app.grid_step, crate::canvas::GridStep::ALL[1]);
    press_key(
        &mut app,
        eframe::egui::Key::N,
        eframe::egui::Modifiers::SHIFT,
    );
    assert_eq!(app.grid_step, crate::canvas::GridStep::ALL[0]);
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
fn status_bar_renders_with_icons() {
    let app = GuiApp::default();
    let mut output =
        eframe::egui::Context::default().run_ui(test_input(), |ui| app.status_contents(ui));
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn view_controls_render_with_icons() {
    let mut app = GuiApp::default();
    let mut output = eframe::egui::Context::default().run_ui(test_input(), |ui| {
        app.zoom_controls(ui);
        app.view_options(ui);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn theme_preference_matches_flag() {
    use eframe::egui::ThemePreference;
    let light = crate::app::view::theme_preference(true);
    let dark = crate::app::view::theme_preference(false);
    assert_eq!(light, ThemePreference::Light);
    assert_eq!(dark, ThemePreference::Dark);
}

#[test]
fn tool_icons_are_distinct() {
    let icons: std::collections::BTreeSet<_> = Tool::ALL.iter().map(|tool| tool.icon()).collect();
    assert_eq!(icons.len(), Tool::ALL.len());
}
