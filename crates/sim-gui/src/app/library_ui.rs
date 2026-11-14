use super::*;

pub(crate) fn category_buttons(ui: &mut egui::Ui, category: &mut Option<Category>) {
    if ui.selectable_label(category.is_none(), "All").clicked() {
        *category = None;
    }
    for value in Category::ALL {
        if ui
            .selectable_label(*category == Some(value), value.label())
            .clicked()
        {
            *category = Some(value);
        }
    }
}

pub(crate) fn library_list<'a>(
    ui: &mut egui::Ui,
    library: &'a [ComponentDefinition],
    search: &str,
    category: Option<Category>,
    hovered: &mut Option<&'a ComponentDefinition>,
    dragged: &mut Option<&'a ComponentDefinition>,
) -> Option<&'a ComponentDefinition> {
    let needle = search.to_lowercase();
    for definition in library {
        if !library_matches(definition, &needle, category) {
            continue;
        }
        if library_item(ui, definition, hovered, dragged) {
            return Some(definition);
        }
    }
    None
}

pub(crate) fn library_matches(
    definition: &ComponentDefinition,
    needle: &str,
    category: Option<Category>,
) -> bool {
    if category.is_some_and(|value| value != definition.category) {
        return false;
    }
    needle.is_empty() || definition.name.to_lowercase().contains(needle)
}

pub(crate) fn library_item<'a>(
    ui: &mut egui::Ui,
    definition: &'a ComponentDefinition,
    hovered: &mut Option<&'a ComponentDefinition>,
    dragged: &mut Option<&'a ComponentDefinition>,
) -> bool {
    let text = format!("{}  {}", definition.name, definition.default_value);
    let response = ui
        .selectable_label(false, text)
        .interact(egui::Sense::click_and_drag());
    if response.hovered() {
        *hovered = Some(definition);
    }
    if response.drag_started() {
        *dragged = Some(definition);
    }
    response.clicked()
}

pub(crate) fn library_preview(ui: &mut egui::Ui, definition: &ComponentDefinition) {
    ui.separator();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), 96.0),
        egui::Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, 4.0, egui::Color32::from_rgb(22, 26, 32));
    scene::draw_library_preview(ui.painter(), rect.shrink(8.0), definition);
    ui.label(format!(
        "{}  {} pins",
        definition.default_value,
        definition.pins.len()
    ));
}
