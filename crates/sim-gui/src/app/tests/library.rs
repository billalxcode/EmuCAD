use super::*;

#[test]
fn recent_components_are_bounded_and_deduplicated() {
    let definitions = crate::library::builtin();
    let mut app = GuiApp::default();
    for definition in definitions.iter().take(6) {
        app.remember_component(definition);
    }
    assert_eq!(app.recent_components.len(), 5);
    assert_eq!(app.recent_components[0].id, definitions[5].id);
    app.remember_component(&definitions[0]);
    assert_eq!(app.recent_components[0].id, definitions[0].id);
}
