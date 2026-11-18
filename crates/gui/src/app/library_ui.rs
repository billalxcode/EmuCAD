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
    let mut chosen = None;
    let height = (ui.available_height() - 140.0).max(160.0);
    egui::ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, true])
        .show_rows(
            ui,
            crate::design::ROW_HEIGHT,
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
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), crate::design::ROW_HEIGHT),
        egui::Sense::click_and_drag(),
    );
    if response.hovered() {
        *hovered = Some(definition);
    }
    if response.drag_started() {
        *dragged = Some(definition);
    }
    paint_library_row(ui, rect, definition, response.hovered());
    response.clicked()
}

pub(crate) fn library_value(definition: &ComponentDefinition) -> Option<&'static str> {
    let value = definition.default_value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case(definition.name) {
        return None;
    }
    Some(value)
}

fn paint_library_row(
    ui: &egui::Ui,
    rect: egui::Rect,
    definition: &ComponentDefinition,
    hovered: bool,
) {
    let colors = crate::design::palette(ui.visuals().dark_mode);
    let inner = rect.shrink2(egui::vec2(8.0, 0.0));
    paint_row_background(ui, rect, hovered, colors);
    paint_row_content(ui, inner, definition, colors);
    paint_row_separator(ui, rect, colors);
}

fn paint_row_content(
    ui: &egui::Ui,
    inner: egui::Rect,
    definition: &ComponentDefinition,
    colors: crate::design::Palette,
) {
    let value = library_value(definition);
    let value_width = row_value_width(ui, value, colors);
    let width = inner.width() - value_width;
    paint_row_name(ui, inner, definition.name, width, colors);
    if let Some(value) = value {
        paint_row_value(ui, inner, value, colors);
    }
}

fn paint_row_background(
    ui: &egui::Ui,
    rect: egui::Rect,
    hovered: bool,
    colors: crate::design::Palette,
) {
    if !hovered {
        return;
    }
    ui.painter()
        .rect_filled(rect.shrink(1.0), 4.0, colors.surface_hover);
}

fn row_value_width(ui: &egui::Ui, value: Option<&str>, colors: crate::design::Palette) -> f32 {
    let Some(value) = value else {
        return 0.0;
    };
    ui.painter()
        .layout_no_wrap(
            value.to_owned(),
            crate::design::mono(11.0),
            colors.text_muted,
        )
        .size()
        .x
        + 12.0
}

fn paint_row_name(
    ui: &egui::Ui,
    inner: egui::Rect,
    name: &str,
    width: f32,
    colors: crate::design::Palette,
) {
    let font = crate::design::medium(14.0);
    let galley = ui
        .painter()
        .layout_job(text_job(name, font, colors.text, width.max(24.0)));
    let pos = egui::pos2(inner.left(), inner.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, colors.text);
}

fn paint_row_value(ui: &egui::Ui, inner: egui::Rect, value: &str, colors: crate::design::Palette) {
    ui.painter().text(
        inner.right_center(),
        egui::Align2::RIGHT_CENTER,
        value,
        crate::design::mono(11.0),
        colors.text_muted,
    );
}

fn paint_row_separator(ui: &egui::Ui, rect: egui::Rect, colors: crate::design::Palette) {
    let line_y = rect.bottom() - 0.5;
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 8.0, line_y),
            egui::pos2(rect.right() - 8.0, line_y),
        ],
        egui::Stroke::new(1.0, colors.faint),
    );
}

fn text_job(
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id: font,
            color,
            ..Default::default()
        },
    );
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('.');
    job
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
    let details = library_value(definition).map_or_else(
        || format!("{} - {} pins", definition.name, definition.pins.len()),
        |value| {
            format!(
                "{} - {} - {} pins",
                definition.name,
                value,
                definition.pins.len()
            )
        },
    );
    ui.label(details);
}
