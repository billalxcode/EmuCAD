use super::*;
use crate::icons::IconKind;

pub(crate) fn show_wire_class_combo(
    ui: &mut egui::Ui,
    salt: &str,
    current: Option<NetClass>,
    action: &mut Option<InspectorAction>,
) {
    let choice = class_choice(ui, salt, current);
    if choice != current {
        if let Some(class) = choice {
            *action = Some(InspectorAction::SetClass(class));
        }
    }
}

fn class_choice(ui: &mut egui::Ui, salt: &str, current: Option<NetClass>) -> Option<NetClass> {
    let mut choice = current;
    ui.horizontal(|ui| {
        icon_slot(ui, IconKind::Route);
        class_combo(ui, salt, &mut choice);
    });
    choice
}

fn class_combo(ui: &mut egui::Ui, salt: &str, choice: &mut Option<NetClass>) {
    egui::ComboBox::from_id_salt(salt)
        .selected_text((*choice).map_or("Mixed", NetClass::label))
        .show_ui(ui, |ui| {
            for class in NetClass::ALL {
                ui.selectable_value(choice, Some(class), class.label());
            }
        });
}

pub(crate) fn wire_rgb(wire: &crate::model::Wire, dark: bool) -> [u8; 3] {
    let [red, green, blue, _] = scene::wire_color(wire, dark).to_srgba_unmultiplied();
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
        icon_slot(ui, IconKind::Palette);
        ui.label("Color");
        if ui.color_edit_button_srgb(&mut rgb).changed() {
            *action = Some(InspectorAction::SetColor(Some(rgb)));
        }
        let reset = icon_button(IconKind::Close, "Reset");
        if ui.add_enabled(has_override, reset).clicked() {
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
        icon_slot(ui, IconKind::Pencil);
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        let set = icon_button(IconKind::Pencil, "Set");
        if ui.add(set).clicked() {
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
