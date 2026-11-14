use super::*;

pub(crate) fn show_wire_class_combo(
    ui: &mut egui::Ui,
    salt: &str,
    current: Option<NetClass>,
    action: &mut Option<InspectorAction>,
) {
    let mut choice = current;
    egui::ComboBox::from_id_salt(salt)
        .selected_text(current.map_or("Mixed", NetClass::label))
        .show_ui(ui, |ui| {
            for class in NetClass::ALL {
                ui.selectable_value(&mut choice, Some(class), class.label());
            }
        });
    if choice != current {
        if let Some(class) = choice {
            *action = Some(InspectorAction::SetClass(class));
        }
    }
}

pub(crate) fn wire_rgb(wire: &crate::model::Wire) -> [u8; 3] {
    let [red, green, blue, _] = scene::wire_color(wire).to_srgba_unmultiplied();
    [red, green, blue]
}

pub(crate) fn show_wire_color_button(
    ui: &mut egui::Ui,
    current: [u8; 3],
    has_override: bool,
    action: &mut Option<InspectorAction>,
) {
    let mut rgb = current;
    ui.horizontal(|ui| {
        ui.label("Color");
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            *action = Some(InspectorAction::SetColor(Some(rgb)));
        }
        if ui
            .add_enabled(has_override, egui::Button::new("Reset"))
            .clicked()
        {
            *action = Some(InspectorAction::SetColor(None));
        }
    });
}

pub(crate) fn show_wire_label_editor(
    ui: &mut egui::Ui,
    current: Option<&str>,
    action: &mut Option<InspectorAction>,
) {
    let mut label = current.unwrap_or("").to_string();
    ui.horizontal(|ui| {
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        if ui.button("Set").clicked() {
            *action = Some(InspectorAction::SetLabel(normalize_label(label)));
        }
    });
}

pub(crate) fn normalize_label(label: String) -> Option<String> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}
