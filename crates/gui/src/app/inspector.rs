use super::*;
use crate::icons::IconKind;
use crate::netlist::ParameterField;

impl GuiApp {
    pub(super) fn inspector_contents(
        &self,
        ui: &mut egui::Ui,
        action: &mut Option<InspectorAction>,
    ) {
        if let Some(id) = self.selection.single_wire() {
            self.wire_inspector(ui, action, id);
            return;
        }
        if let Some((component, definition)) = self.selected_pair() {
            self.selected_inspector(ui, component, definition, action);
            return;
        }
        if self.selection.is_empty() {
            ui.label("No selection");
            return;
        }
        self.multi_inspector(ui, action);
    }

    fn selected_inspector(
        &self,
        ui: &mut egui::Ui,
        component: &ComponentInstance,
        definition: &ComponentDefinition,
        action: &mut Option<InspectorAction>,
    ) {
        show_component_info(ui, component, definition, action);
        show_component_value(ui, component, definition, action);
        show_component_actions(ui, action);
    }

    pub(super) fn multi_inspector(&self, ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
        let components = self.selection.components().len();
        let wires = self.selection.wires().len();
        ui.label(format!("{components} components, {wires} wires"));
        if components > 0 {
            show_component_actions(ui, action);
        }
        if wires > 0 {
            self.multi_wire_controls(ui, action);
        }
    }

    pub(super) fn multi_wire_controls(
        &self,
        ui: &mut egui::Ui,
        action: &mut Option<InspectorAction>,
    ) {
        show_wire_class_combo(ui, "multi_wire_class", self.selection_wire_class(), action);
        if let Some(rgb) = self.selection_wire_rgb(ui.visuals().dark_mode) {
            show_wire_color_button(ui, rgb, self.selection_has_color_override(), action);
        }
        let delete = icon_button(IconKind::Trash, "Delete");
        if self.selection.components().is_empty() && ui.add(delete).clicked() {
            *action = Some(InspectorAction::Delete);
        }
    }

    pub(super) fn selection_wire_class(&self) -> Option<NetClass> {
        let mut classes = self.selection.wires().iter().filter_map(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .map(|wire| wire.class)
        });
        let first = classes.next()?;
        classes.all(|class| class == first).then_some(first)
    }

    pub(super) fn selection_wire_rgb(&self, dark: bool) -> Option<[u8; 3]> {
        self.selection.wires().iter().find_map(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .map(|wire| wire_rgb(wire, dark))
        })
    }

    pub(super) fn selection_wire_label(&self) -> String {
        self.selection
            .wires()
            .iter()
            .find_map(|id| {
                self.document
                    .wires()
                    .iter()
                    .find(|wire| wire.id == *id)
                    .and_then(|wire| wire.label.clone())
            })
            .unwrap_or_default()
    }

    pub(super) fn context_net_state(&self) -> ContextNetState {
        ContextNetState {
            class: self.selection_wire_class(),
            has_override: self.selection_has_color_override(),
            label: self.selection_wire_label(),
        }
    }

    pub(super) fn selection_has_color_override(&self) -> bool {
        self.selection.wires().iter().any(|id| {
            self.document
                .wires()
                .iter()
                .find(|wire| wire.id == *id)
                .is_some_and(|wire| wire.color_override.is_some())
        })
    }

    pub(super) fn wire_inspector(
        &self,
        ui: &mut egui::Ui,
        action: &mut Option<InspectorAction>,
        id: WireId,
    ) {
        let Some(wire) = self.document.wires().iter().find(|wire| wire.id == id) else {
            ui.label("Wire missing");
            return;
        };
        ui.label(format!("Wire {}", wire.id.0));
        show_wire_class_combo(ui, "inspector_wire_class", Some(wire.class), action);
        ui.label(format!("Vertices: {}", wire.points.len()));
        show_wire_color_button(
            ui,
            wire_rgb(wire, ui.visuals().dark_mode),
            wire.color_override.is_some(),
            action,
        );
        show_wire_label_editor(ui, wire.label.as_deref(), action);
        let delete = icon_button(IconKind::Trash, "Delete");
        if ui.add(delete).clicked() {
            *action = Some(InspectorAction::Delete);
        }
    }

    pub(super) fn selected_pair(&self) -> Option<(&ComponentInstance, &ComponentDefinition)> {
        let id = self.selection.single_component()?;
        let component = self.document.component(id)?;
        let definition = library::find(component.definition)?;
        Some((component, definition))
    }

    pub(super) fn copy_selection(&mut self) {
        if self.selection.components().is_empty() {
            self.message = "Nothing to copy".to_string();
            return;
        }
        self.clipboard = Clipboard::capture(
            &self.document,
            self.selection.components(),
            self.selection.wires(),
        );
        if self.clipboard.is_empty() {
            self.message = "Nothing to copy".to_string();
            return;
        }
        self.message = format!(
            "Copied {} components, {} wires",
            self.selection.components().len(),
            self.selection.wires().len()
        );
    }

    pub(super) fn cut_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        self.copy_selection();
        self.delete_selection();
    }

    pub(super) fn paste_selection(&mut self) {
        let offset = self.paste_offset();
        let before = self.document.clone();
        match self
            .clipboard
            .paste(&mut self.document, self.library, offset)
        {
            Ok(result) => self.after_paste(result, before),
            Err(error) => self.message = error,
        }
    }

    pub(super) fn duplicate_selection(&mut self) {
        let clipboard = Clipboard::capture(
            &self.document,
            self.selection.components(),
            self.selection.wires(),
        );
        if clipboard.is_empty() {
            self.message = "Nothing to duplicate".to_string();
            return;
        }
        let before = self.document.clone();
        match clipboard.paste(&mut self.document, self.library, GridPos::new(2, 2)) {
            Ok(result) => self.after_paste(result, before),
            Err(error) => self.message = error,
        }
    }

    pub(super) fn after_paste(&mut self, result: clipboard::PasteResult, before: Document) {
        let components = result.components.len();
        let wires = result.wires.len();
        self.history.record(&before);
        self.selection.apply(result.components, result.wires, false);
        self.mark_dirty();
        self.message = format!("Pasted {components} components, {wires} wires");
    }

    pub(super) fn paste_offset(&self) -> GridPos {
        let Some(world) = self.pointer_world else {
            return GridPos::new(2, 2);
        };
        let offset = self
            .snap_world(world)
            .point
            .subtract(self.clipboard.origin());
        if offset == GridPos::new(0, 0) {
            GridPos::new(2, 2)
        } else {
            offset
        }
    }

    pub(super) fn push_undo(&mut self) {
        self.history.record(&self.document);
    }

    pub(super) fn undo(&mut self) {
        let Some(previous) = self.history.undo(&self.document) else {
            return;
        };
        self.document = previous;
        self.reset_transient();
        self.message = "Undo".to_string();
        self.mark_dirty();
    }

    pub(super) fn redo(&mut self) {
        let Some(next) = self.history.redo(&self.document) else {
            return;
        };
        self.document = next;
        self.reset_transient();
        self.message = "Redo".to_string();
        self.mark_dirty();
    }

    pub(super) fn reset_transient(&mut self) {
        self.selection.clear();
        self.box_start = None;
        self.box_current = None;
        self.box_additive = false;
        self.pending_delete = false;
        self.dragging = None;
        self.dragging_segment = None;
        self.drag_history_pending = false;
        self.placing = None;
        self.drag_place = false;
        self.wire_points.clear();
    }

    pub(super) fn delete_selection(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        if self.needs_delete_confirmation() {
            self.pending_delete = true;
            self.context_menu = None;
            self.palette.close();
            return;
        }
        self.delete_selection_now();
    }

    pub(super) fn needs_delete_confirmation(&self) -> bool {
        if self.selection.len() > 5 {
            return true;
        }
        self.selection
            .components()
            .iter()
            .any(|id| self.document.component_has_wire(self.library, *id))
    }

    pub(super) fn delete_selection_now(&mut self) {
        self.push_undo();
        let wires: Vec<_> = self.selection.wires().iter().copied().collect();
        let components: Vec<_> = self.selection.components().iter().copied().collect();
        for id in wires {
            self.document.remove_wire(id);
        }
        for id in components {
            self.document.remove(id);
        }
        self.selection.clear();
        self.mark_dirty();
    }

    pub(super) fn apply_inspector(&mut self, action: InspectorAction) {
        match action {
            InspectorAction::Rotate => self.rotate(),
            InspectorAction::Mirror => self.mirror(),
            InspectorAction::Delete => self.delete_selection(),
            InspectorAction::SetClass(class) => self.set_wire_class(class),
            InspectorAction::SetColor(color) => self.set_wire_color_override(color),
            InspectorAction::SetLabel(label) => self.set_wire_label(label),
            InspectorAction::SetParameter { name, value } => {
                self.set_component_parameter(name, value)
            }
            InspectorAction::SetPinMode { pin, mode } => self.set_pin_mode(pin, mode),
        }
    }

    pub(super) fn set_pin_mode(&mut self, pin: u8, mode: McuPinMode) {
        let Some(id) = self.selection.single_component() else {
            return;
        };
        let current = self
            .document
            .component(id)
            .and_then(|component| component.pin_modes.get(&pin).copied());
        if current == Some(mode) {
            return;
        }
        self.push_undo();
        let Some(component) = self.document.component_mut(id) else {
            return;
        };
        component.pin_modes.insert(pin, mode);
        self.mark_dirty();
        self.message = format!("Pin {pin} mode {}", mode.label());
    }

    pub(super) fn set_component_parameter(&mut self, name: &'static str, value: f64) {
        let Some((id, field)) = self.selected_parameter_field(name) else {
            return;
        };
        if !field.accepts(value) {
            self.message = format!("Invalid value for {name}");
            return;
        }
        if self.current_parameter(id, &field) == value && !field.primary {
            return;
        }
        self.push_undo();
        self.write_parameter(id, name, value, &field);
        self.mark_dirty();
        self.message = "Value updated; rerun DC".to_string();
    }

    fn selected_parameter_field(
        &self,
        name: &str,
    ) -> Option<(crate::model::ComponentId, ParameterField)> {
        let id = self.selection.single_component()?;
        let component = self.document.component(id)?;
        let definition = crate::library::find(component.definition)?;
        let field = crate::netlist::parameter_fields(definition)
            .into_iter()
            .find(|field| field.spec.name == name)?;
        Some((id, field))
    }

    fn current_parameter(&self, id: crate::model::ComponentId, field: &ParameterField) -> f64 {
        self.document
            .component(id)
            .and_then(|component| component.parameters.get(field.spec.name).copied())
            .unwrap_or(field.default)
    }

    fn write_parameter(
        &mut self,
        id: crate::model::ComponentId,
        name: &str,
        value: f64,
        field: &ParameterField,
    ) {
        let Some(component) = self.document.component_mut(id) else {
            return;
        };
        component.parameters.insert(name.to_string(), value);
        if field.primary {
            component.value = format_parameter(value, field.spec.unit);
        }
    }

    pub(super) fn set_wire_label(&mut self, label: Option<String>) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        let mut changed = false;
        for id in ids {
            changed |= self.document.set_wire_label(id, label.clone());
        }
        if !changed {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = match label {
            Some(_) => "Wire label updated".to_string(),
            None => "Wire label cleared".to_string(),
        };
    }

    pub(super) fn set_wire_color_override(&mut self, color: Option<[u8; 3]>) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        if !self.apply_wire_color(&ids, color) {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = wire_color_message(color.is_some());
    }

    pub(super) fn apply_wire_color(&mut self, ids: &[WireId], color: Option<[u8; 3]>) -> bool {
        let mut changed = false;
        for &id in ids {
            changed |= self.document.set_wire_color(id, color);
        }
        changed
    }

    pub(super) fn set_wire_class(&mut self, class: NetClass) {
        let ids: Vec<_> = self.selection.wires().iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        let before = self.document.clone();
        let mut changed = false;
        for id in ids {
            changed |= self.document.set_wire_class(id, class);
        }
        if !changed {
            return;
        }
        self.history.record(&before);
        self.mark_dirty();
        self.message = format!("Net class {}", class.label());
    }
}

fn format_parameter(value: f64, unit: &str) -> String {
    if unit.is_empty() {
        return format!("{value}");
    }
    format!("{value} {unit}")
}
