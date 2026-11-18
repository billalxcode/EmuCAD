use super::*;

pub(crate) fn show_net_legend(ui: &mut egui::Ui) {
    for class in NetClass::ALL {
        ui.horizontal(|ui| {
            legend_swatch(ui, class);
            ui.label(class.label());
        });
    }
}

pub(crate) fn legend_swatch(ui: &mut egui::Ui, class: NetClass) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 8.0), egui::Sense::hover());
    let y = rect.center().y;
    let points = [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)];
    let stroke = egui::Stroke::new(2.0, scene::net_color(class, ui.visuals().dark_mode));
    if let Some((dash, gap)) = scene::wire_dash(class) {
        ui.painter()
            .extend(egui::Shape::dashed_line(&points, stroke, dash, gap));
        return;
    }
    ui.painter().line_segment(points, stroke);
}
