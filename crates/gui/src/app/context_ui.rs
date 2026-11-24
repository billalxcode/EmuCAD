use super::*;
use crate::icons::IconKind;

pub(crate) fn context_closes(ui: &egui::Ui, response: &egui::Response, has_action: bool) -> bool {
    if has_action {
        return false;
    }
    ui.input(|input| {
        input.key_pressed(egui::Key::Escape)
            || (!input.pointer.secondary_clicked()
                && input.pointer.any_click()
                && !response.contains_pointer())
    })
}

pub(crate) fn show_context_area(
    ctx: &egui::Context,
    position: egui::Pos2,
    selection: &Selection,
    clipboard_empty: bool,
    net: ContextNetState,
) -> (Option<ContextAction>, egui::Response) {
    let mut action = None;
    let area = egui::Area::new(egui::Id::unique("canvas-context"))
        .fixed_pos(position)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(190.0);
                context_buttons(ui, selection, clipboard_empty, &net, &mut action);
            });
        });
    (action, area.response)
}

pub(crate) fn context_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    clipboard_empty: bool,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    context_edit_buttons(ui, selection, clipboard_empty, action);
    ui.separator();
    context_transform_buttons(ui, selection, action);
    if !selection.wires().is_empty() {
        ui.separator();
        context_net_controls(ui, net, action);
    }
    ui.separator();
    context_tool_buttons(ui, action);
}

pub(crate) fn context_net_controls(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    icon_heading(ui, IconKind::Junction, "Net");
    context_class_combo(ui, net.class, action);
    context_color_presets(ui, net, action);
    context_label_editor(ui, net, action);
}

pub(crate) fn context_color_presets(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    ui.horizontal_wrapped(|ui| {
        icon_slot(ui, IconKind::Palette);
        ui.label("Color");
        for &(name, rgb) in &CONTEXT_COLORS {
            if context_color_button(ui, name, rgb).clicked() {
                *action = Some(ContextAction::SetColor(Some(rgb)));
            }
        }
        let reset = icon_button(IconKind::Close, "Reset");
        if ui.add_enabled(net.has_override, reset).clicked() {
            *action = Some(ContextAction::SetColor(None));
        }
    });
}

pub(crate) fn context_color_button(ui: &mut egui::Ui, name: &str, rgb: [u8; 3]) -> egui::Response {
    let button = egui::Button::new(" ")
        .fill(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
        .min_size(egui::vec2(16.0, 16.0));
    ui.add(button).on_hover_text(name)
}

pub(crate) fn context_label_editor(
    ui: &mut egui::Ui,
    net: &ContextNetState,
    action: &mut Option<ContextAction>,
) {
    let mut label = net.label.clone();
    ui.horizontal(|ui| {
        icon_slot(ui, IconKind::Pencil);
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        let set = icon_button(IconKind::Pencil, "Set");
        if ui.add(set).clicked() {
            *action = Some(ContextAction::SetLabel(normalize_label(label)));
        }
    });
}

pub(crate) fn context_class_combo(
    ui: &mut egui::Ui,
    current: Option<NetClass>,
    action: &mut Option<ContextAction>,
) {
    let mut choice = current;
    ui.horizontal(|ui| {
        icon_slot(ui, IconKind::Route);
        egui::ComboBox::from_id_salt("context_net_class")
            .selected_text(class_combo_text(current))
            .show_ui(ui, |ui| {
                for class in NetClass::ALL {
                    ui.selectable_value(&mut choice, Some(class), class.label());
                }
            });
    });
    if let Some(class) = choice.filter(|value| Some(*value) != current) {
        *action = Some(ContextAction::SetClass(class));
    }
}

pub(crate) fn class_combo_text(current: Option<NetClass>) -> String {
    current.map_or_else(
        || "Net class: mixed".to_string(),
        |class| format!("Net class: {}", class.label()),
    )
}

pub(crate) fn context_edit_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    clipboard_empty: bool,
    action: &mut Option<ContextAction>,
) {
    let any = !selection.is_empty();
    context_button(ui, any, IconKind::Copy, "Copy", ContextAction::Copy, action);
    context_button(ui, any, IconKind::Cut, "Cut", ContextAction::Cut, action);
    let paste = !clipboard_empty;
    context_button(
        ui,
        paste,
        IconKind::Clipboard,
        "Paste",
        ContextAction::Paste,
        action,
    );
    context_button(
        ui,
        any,
        IconKind::Copy,
        "Duplicate",
        ContextAction::Duplicate,
        action,
    );
    context_button(
        ui,
        any,
        IconKind::Trash,
        "Delete",
        ContextAction::Delete,
        action,
    );
    context_button(
        ui,
        any,
        IconKind::Settings,
        "Properties",
        ContextAction::Properties,
        action,
    );
}

pub(crate) fn context_button(
    ui: &mut egui::Ui,
    enabled: bool,
    icon: IconKind,
    label: &str,
    value: ContextAction,
    action: &mut Option<ContextAction>,
) {
    let button = icon_button(icon, label);
    if ui.add_enabled(enabled, button).clicked() {
        *action = Some(value);
    }
}

pub(crate) fn context_transform_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    action: &mut Option<ContextAction>,
) {
    let components = !selection.components().is_empty();
    let rotate = icon_button(IconKind::Rotate, "Rotate");
    if ui.add_enabled(components, rotate).clicked() {
        *action = Some(ContextAction::Rotate);
    }
    let mirror = icon_button(IconKind::FlipHorizontal, "Mirror");
    if ui.add_enabled(components, mirror).clicked() {
        *action = Some(ContextAction::Mirror);
    }
}

pub(crate) fn context_tool_buttons(ui: &mut egui::Ui, action: &mut Option<ContextAction>) {
    context_tool_button(
        ui,
        IconKind::Chart,
        "Digital graph",
        ContextAction::Graph,
        action,
    );
    context_tool_button(
        ui,
        IconKind::Crosshair,
        "Probe PB5",
        ContextAction::ProbePb5,
        action,
    );
    context_tool_button(
        ui,
        IconKind::Crosshair,
        "Probe LED",
        ContextAction::ProbeLed,
        action,
    );
    context_tool_button(
        ui,
        IconKind::Pointer,
        "Select tool",
        ContextAction::SelectTool,
        action,
    );
    context_tool_button(
        ui,
        IconKind::Route,
        "Wire tool",
        ContextAction::WireTool,
        action,
    );
}

fn context_tool_button(
    ui: &mut egui::Ui,
    icon: IconKind,
    label: &str,
    value: ContextAction,
    action: &mut Option<ContextAction>,
) {
    if ui.add(icon_button(icon, label)).clicked() {
        *action = Some(value);
    }
}
