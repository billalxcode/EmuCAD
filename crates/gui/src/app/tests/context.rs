use super::*;

fn context_click(
    world: eframe::egui::Pos2,
    screen: eframe::egui::Pos2,
) -> crate::canvas::CanvasState {
    crate::canvas::CanvasState {
        context_click: Some(crate::canvas::ContextClick { screen, world }),
        ..Default::default()
    }
}

#[test]
fn context_click_selects_target() {
    let (mut app, first, second) = context_app();
    app.selection.select_component(second);
    app.open_context_menu(&context_click(
        eframe::egui::pos2(1.0, 1.0),
        eframe::egui::pos2(10.0, 10.0),
    ));
    assert_eq!(app.selection.single_component(), Some(first));
    assert_eq!(app.context_menu, Some(eframe::egui::pos2(10.0, 10.0)));
}

#[test]
fn context_click_empty_clears() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    app.open_context_menu(&context_click(
        eframe::egui::pos2(30.0, 30.0),
        eframe::egui::pos2(20.0, 20.0),
    ));
    assert!(app.selection.is_empty());
}

#[test]
fn cut_action_copies_and_deletes() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    app.run_context_action(crate::app::ContextAction::Cut);
    assert!(app.document.component(first).is_none());
    assert!(!app.clipboard.is_empty());
}

#[test]
fn context_menu_renders_with_icons() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    let mut output = eframe::egui::Context::default().run_ui(test_input(), |ui| {
        let _ = app.context_area(ui, eframe::egui::pos2(10.0, 10.0));
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn inspector_actions_render_with_icons() {
    let mut output = eframe::egui::Context::default().run_ui(test_input(), |ui| {
        let mut action = None;
        crate::app::show_component_actions(ui, &mut action);
    });
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

#[test]
fn every_mcu_mode_has_an_icon() {
    for mode in McuPinMode::ALL {
        assert!(crate::icons::IconKind::ALL.contains(&crate::app::mcu_mode_icon(mode)));
    }
}
