use super::*;

impl GuiApp {
    pub(crate) fn show_context_menu(&mut self, ui: &mut egui::Ui) {
        let Some(position) = self.context_menu else {
            return;
        };
        if self.conflict || self.palette.open {
            self.context_menu = None;
            return;
        }
        let (action, response) = self.context_area(ui, position);
        if context_closes(ui, &response, action.is_some()) {
            self.context_menu = None;
        }
        if let Some(action) = action {
            self.context_menu = None;
            self.run_context_action(action);
        }
    }

    pub(crate) fn context_area(
        &self,
        ui: &egui::Ui,
        position: egui::Pos2,
    ) -> (Option<ContextAction>, egui::Response) {
        show_context_area(
            ui.ctx(),
            position,
            &self.selection,
            self.clipboard.is_empty(),
            self.context_net_state(),
        )
    }

    pub(crate) fn run_context_action(&mut self, action: ContextAction) {
        match action {
            ContextAction::Copy => self.copy_selection(),
            ContextAction::Cut => self.cut_selection(),
            ContextAction::Paste => self.paste_selection(),
            ContextAction::Duplicate => self.duplicate_selection(),
            ContextAction::Delete => self.delete_selection(),
            ContextAction::Rotate => self.rotate(),
            ContextAction::Mirror => self.mirror(),
            ContextAction::Properties => self.inspector_open = true,
            ContextAction::Graph => self.waveform_open = true,
            ContextAction::ProbePb5 => self.open_pb5_probe(),
            ContextAction::ProbeLed => self.open_led_probe(),
            ContextAction::SetClass(class) => self.set_wire_class(class),
            ContextAction::SetColor(color) => self.set_wire_color_override(color),
            ContextAction::SetLabel(label) => self.set_wire_label(label),
            ContextAction::SelectTool => self.set_tool(Tool::Select),
            ContextAction::WireTool => self.set_tool(Tool::Wire),
        }
    }
}
