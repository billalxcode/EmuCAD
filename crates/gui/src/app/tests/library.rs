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

fn assert_passive_filter(definitions: &[ComponentDefinition]) {
    let passive = crate::app::library_ui::filtered_library(
        definitions,
        "",
        Some(crate::model::Category::Passive),
    );
    assert!(passive.len() >= 50);
    assert!(
        passive
            .iter()
            .all(|definition| definition.category == crate::model::Category::Passive)
    );
}

fn assert_zener_filter(definitions: &[ComponentDefinition]) {
    let zener = crate::app::library_ui::filtered_library(definitions, "zener", None);
    assert!(!zener.is_empty());
    assert!(
        zener
            .iter()
            .all(|definition| definition.name.to_lowercase().contains("zener"))
    );
}

#[test]
fn library_filters_catalog_by_category_and_search() {
    let definitions = crate::library::builtin();
    assert_passive_filter(definitions);
    assert_zener_filter(definitions);
}

fn run_library_ui<R>(
    width: f32,
    height: f32,
    mut draw: impl FnMut(&mut eframe::egui::Ui) -> R,
) -> (R, eframe::egui::FullOutput) {
    let context = eframe::egui::Context::default();
    let input = eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(width, height),
        )),
        ..Default::default()
    };
    let mut result = None;
    let mut output = context.run_ui(input, |ui| {
        result = Some(draw(ui));
    });
    output.textures_delta.clear();
    (result.expect("ui result"), output)
}

#[test]
fn library_scroll_renders_catalog() {
    let definitions = crate::library::builtin();
    let mut hovered = None;
    let mut dragged = None;
    let (_, output) = run_library_ui(320.0, 240.0, |ui| {
        let _ = crate::app::library_ui::library_scroll(
            ui,
            definitions,
            "",
            None,
            &mut hovered,
            &mut dragged,
        );
    });
    assert!(!output.shapes.is_empty());
}

#[test]
fn library_window_renders_full_catalog() {
    let app = GuiApp::default();
    let mut open = true;
    let (_, output) = run_library_ui(800.0, 600.0, |ui| {
        let mut state = crate::app::LibraryState {
            search: String::new(),
            category: None,
            chosen: None,
            hovered: None,
            dragged: None,
        };
        app.show_library(ui, &mut open, &mut state);
    });
    assert!(open);
    assert!(!output.shapes.is_empty());
}

#[test]
fn library_value_omits_duplicate_names() {
    let atmega = crate::library::find("atmega328p").expect("atmega");
    let led = crate::library::find("led").expect("led");
    let resistor = crate::library::find("resistor").expect("resistor");
    assert_eq!(crate::app::library_ui::library_value(atmega), None);
    assert_eq!(crate::app::library_ui::library_value(led), None);
    assert_eq!(crate::app::library_ui::library_value(resistor), Some("220"));
}

#[test]
fn every_category_has_a_vendored_icon() {
    for category in crate::model::Category::ALL {
        let icon = crate::app::library_ui::category_icon(category);
        assert!(crate::icons::IconKind::ALL.contains(&icon));
    }
}

#[test]
fn library_rows_report_model_status() {
    use crate::netlist::{ModelStatus, model_status};
    let resistor = crate::library::find("resistor").expect("resistor");
    let gnd = crate::library::find("gnd").expect("gnd");
    let behavioral =
        crate::library::find("source.behavioral.behavioral_voltage_source").expect("behavioral");
    assert_eq!(model_status(resistor), ModelStatus::Device);
    assert_eq!(model_status(gnd), ModelStatus::Marker);
    assert_eq!(model_status(behavioral), ModelStatus::Unsupported);
}

#[test]
fn library_search_field_renders() {
    let (_, output) = run_library_ui(400.0, 100.0, |ui| {
        let mut search = "led".to_string();
        crate::app::library_ui::search_field(ui, &mut search);
    });
    assert!(!output.shapes.is_empty());
}

#[test]
fn category_buttons_render_with_icons() {
    let (_, output) = run_library_ui(600.0, 300.0, |ui| {
        let mut category = Some(crate::model::Category::Passive);
        crate::app::library_ui::category_buttons(ui, &mut category);
    });
    assert!(output.shapes.len() > crate::model::Category::ALL.len());
}

#[test]
fn library_rows_use_a_fixed_height() {
    assert_eq!(crate::design::ROW_HEIGHT, 28.0);
    let definitions = crate::library::builtin();
    let mut hovered = None;
    let mut dragged = None;
    let (_, output) = run_library_ui(320.0, 240.0, |ui| {
        let _ = crate::app::library_ui::library_scroll(
            ui,
            definitions,
            "",
            None,
            &mut hovered,
            &mut dragged,
        );
    });
    assert!(!output.shapes.is_empty());
}
