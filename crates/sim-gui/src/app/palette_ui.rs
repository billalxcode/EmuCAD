use super::*;

pub(crate) fn palette_list(
    ui: &mut egui::Ui,
    running: bool,
    matches: &[Command],
    selected: usize,
    chosen: &mut Option<Command>,
) {
    egui::ScrollArea::vertical()
        .max_height(320.0)
        .show(ui, |ui| {
            for (index, command) in matches.iter().enumerate() {
                let active = index == selected;
                if palette_row(ui, *command, active, running) {
                    *chosen = Some(*command);
                }
            }
        });
}

pub(crate) fn palette_row(
    ui: &mut egui::Ui,
    command: Command,
    active: bool,
    running: bool,
) -> bool {
    ui.selectable_label(active, command.label(running))
        .clicked()
}
