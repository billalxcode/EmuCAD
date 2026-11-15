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

pub(crate) fn library_scroll<'a>(
    ui: &mut egui::Ui,
    library: &'a [ComponentDefinition],
    search: &str,
    category: Option<Category>,
    hovered: &mut Option<&'a ComponentDefinition>,
    dragged: &mut Option<&'a ComponentDefinition>,
) -> Option<&'a ComponentDefinition> {
    let definitions = filtered_library(library, search, category);
    let row_height = ui.spacing().interact_size.y.max(18.0);
    let mut chosen = None;
    let height = (ui.available_height() - 140.0).max(160.0);
    egui::ScrollArea::vertical().max_height(height).show_rows(
        ui,
        row_height,
        definitions.len(),
        |ui, range| {
            for index in range {
                if library_item(ui, definitions[index], hovered, dragged) {
                    chosen = Some(definitions[index]);
                }
            }
        },
    );
    chosen
}

pub(crate) fn filtered_library<'a>(
    library: &'a [ComponentDefinition],
    search: &str,
    category: Option<Category>,
) -> Vec<&'a ComponentDefinition> {
    let needle = search.to_lowercase();
    library
        .iter()
        .filter(|definition| library_matches(definition, &needle, category))
        .collect()
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
    let colors = crate::design::palette(ui.visuals().dark_mode);
    ui.painter().rect_filled(rect, 4.0, colors.surface);
    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, colors.border),
        egui::StrokeKind::Middle,
    );
    scene::draw_library_preview(ui.painter(), rect.shrink(8.0), definition);
    ui.label(format!(
        "{}  {} pins",
        definition.default_value,
        definition.pins.len()
    ));
}
