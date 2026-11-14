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
