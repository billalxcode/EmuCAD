use super::*;

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
        ui.label("Color");
        for &(name, rgb) in &CONTEXT_COLORS {
            if context_color_button(ui, name, rgb).clicked() {
                *action = Some(ContextAction::SetColor(Some(rgb)));
            }
        }
        if ui
            .add_enabled(net.has_override, egui::Button::new("Reset"))
            .clicked()
        {
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
        ui.label("Label");
        ui.text_edit_singleline(&mut label);
        if ui.button("Set").clicked() {
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
    egui::ComboBox::from_id_salt("context_net_class")
        .selected_text(class_combo_text(current))
        .show_ui(ui, |ui| {
            for class in NetClass::ALL {
                ui.selectable_value(&mut choice, Some(class), class.label());
            }
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
    context_button(ui, any, "Copy", ContextAction::Copy, action);
    context_button(ui, !clipboard_empty, "Paste", ContextAction::Paste, action);
    context_button(ui, any, "Duplicate", ContextAction::Duplicate, action);
    context_button(ui, any, "Delete", ContextAction::Delete, action);
    context_button(ui, any, "Properties", ContextAction::Properties, action);
}

pub(crate) fn context_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    value: ContextAction,
    action: &mut Option<ContextAction>,
) {
    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
        *action = Some(value);
    }
}

pub(crate) fn context_transform_buttons(
    ui: &mut egui::Ui,
    selection: &Selection,
    action: &mut Option<ContextAction>,
) {
    let components = !selection.components().is_empty();
    if ui
        .add_enabled(components, egui::Button::new("Rotate"))
        .clicked()
    {
        *action = Some(ContextAction::Rotate);
    }
    if ui
        .add_enabled(components, egui::Button::new("Mirror"))
        .clicked()
    {
        *action = Some(ContextAction::Mirror);
    }
}

pub(crate) fn context_tool_buttons(ui: &mut egui::Ui, action: &mut Option<ContextAction>) {
    if ui.button("Digital graph").clicked() {
        *action = Some(ContextAction::Graph);
    }
    if ui.button("Probe PB5").clicked() {
        *action = Some(ContextAction::ProbePb5);
    }
    if ui.button("Probe LED").clicked() {
        *action = Some(ContextAction::ProbeLed);
    }
    if ui.button("Select tool").clicked() {
        *action = Some(ContextAction::SelectTool);
    }
    if ui.button("Wire tool").clicked() {
        *action = Some(ContextAction::WireTool);
    }
}
