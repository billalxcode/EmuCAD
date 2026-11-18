use super::*;

pub(crate) fn show_uart_contents(
    ui: &mut egui::Ui,
    log: &UartLog,
    input: &mut String,
    clear: &mut bool,
    send: &mut bool,
) {
    show_uart_toolbar(ui, log.len(), clear);
    show_uart_input(ui, input, send);
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| show_uart_text(ui, log.text()));
}

pub(crate) fn show_uart_toolbar(ui: &mut egui::Ui, len: usize, clear: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("Clear").clicked() {
            *clear = true;
        }
        ui.label(format!("{len} bytes"));
    });
}

pub(crate) fn show_uart_input(ui: &mut egui::Ui, input: &mut String, send: &mut bool) {
    ui.horizontal(|ui| {
        let response = ui.text_edit_singleline(input);
        let enter = response.lost_focus() && ui.input(|state| state.key_pressed(egui::Key::Enter));
        if ui.button("Send").clicked() || enter {
            *send = true;
        }
    });
}

pub(crate) fn show_uart_text(ui: &mut egui::Ui, text: &str) {
    if text.is_empty() {
        ui.label("No UART output");
        return;
    }
    let label = egui::RichText::new(text).monospace();
    ui.add(egui::Label::new(label).wrap());
}
