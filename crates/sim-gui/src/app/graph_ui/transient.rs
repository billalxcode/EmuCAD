use super::*;

const SERIES_COLORS: [egui::Color32; 8] = [
    egui::Color32::from_rgb(86, 129, 255),
    egui::Color32::from_rgb(14, 175, 98),
    egui::Color32::from_rgb(224, 100, 0),
    egui::Color32::from_rgb(255, 107, 107),
    egui::Color32::from_rgb(190, 120, 255),
    egui::Color32::from_rgb(0, 190, 190),
    egui::Color32::from_rgb(230, 200, 0),
    egui::Color32::from_rgb(255, 130, 200),
];

pub(crate) fn show_transient(
    ui: &mut egui::Ui,
    trace: &crate::transient::TransientTrace,
    selected: &mut BTreeSet<usize>,
) {
    ui.label(transient_header(trace, selected.len()));
    select_nets(ui, trace, selected);
    let width = ui.available_width().max(320.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 220.0), egui::Sense::hover());
    let plot = response.rect.shrink2(egui::vec2(46.0, 16.0));
    paint_transient(&painter, response.rect, plot, trace, selected);
}

fn transient_header(trace: &crate::transient::TransientTrace, selected: usize) -> String {
    format!(
        "{} samples  dt {:.3} ms  duration {:.3} ms  {} nets",
        trace.values.len(),
        trace.dt * 1e3,
        trace.duration() * 1e3,
        selected
    )
}

fn select_nets(
    ui: &mut egui::Ui,
    trace: &crate::transient::TransientTrace,
    selected: &mut BTreeSet<usize>,
) {
    ui.horizontal_wrapped(|ui| {
        for (index, name) in trace.nodes.iter().enumerate() {
            let mut checked = selected.contains(&index);
            if !ui.checkbox(&mut checked, name).changed() {
                continue;
            }
            if checked && selected.len() < crate::probe::MAX_SERIES {
                selected.insert(index);
            }
            if !checked {
                selected.remove(&index);
            }
        }
    });
}

fn paint_transient(
    painter: &egui::Painter,
    rect: egui::Rect,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    selected: &BTreeSet<usize>,
) {
    let palette = crate::design::palette_for(painter.ctx());
    painter.rect_filled(rect, 4.0, palette.graph_bg);
    let Some(range) = value_range(trace, selected) else {
        paint_empty(painter, plot, palette);
        return;
    };
    paint_grid(painter, plot, range);
    paint_time_labels(painter, plot, trace.duration());
    paint_series_set(painter, plot, trace, selected, range);
    paint_legend(painter, plot, trace, selected);
}

fn paint_empty(painter: &egui::Painter, plot: egui::Rect, palette: crate::design::Palette) {
    painter.text(
        plot.center(),
        egui::Align2::CENTER_CENTER,
        "Select a net",
        crate::design::mono(11.0),
        palette.text_muted,
    );
}

fn paint_series_set(
    painter: &egui::Painter,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    selected: &BTreeSet<usize>,
    range: (f64, f64),
) {
    for (order, &node) in selected.iter().enumerate() {
        paint_series(
            painter,
            plot,
            trace,
            node,
            range,
            SERIES_COLORS[order % SERIES_COLORS.len()],
        );
    }
}

fn value_range(
    trace: &crate::transient::TransientTrace,
    selected: &BTreeSet<usize>,
) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found = false;
    for &node in selected {
        for values in &trace.values {
            let value = values[node];
            if !value.is_finite() {
                continue;
            }
            min = min.min(value);
            max = max.max(value);
            found = true;
        }
    }
    found.then_some(padded_range(min, max))
}

fn padded_range(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < 1e-9 {
        return (min - 0.5, max + 0.5);
    }
    let margin = (max - min) * 0.05;
    (min - margin, max + margin)
}

fn paint_grid(painter: &egui::Painter, plot: egui::Rect, range: (f64, f64)) {
    let palette = crate::design::palette_for(painter.ctx());
    let stroke = egui::Stroke::new(1.0, palette.graph_grid);
    for step in 0..=4 {
        let y = plot.bottom() - plot.height() * step as f32 / 4.0;
        painter.line_segment(
            [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
            stroke,
        );
        let value = range.0 + (range.1 - range.0) * f64::from(step) / 4.0;
        painter.text(
            egui::pos2(plot.left() - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            format_value(value),
            crate::design::mono(10.0),
            palette.text_muted,
        );
    }
}

fn format_value(value: f64) -> String {
    if value.abs() >= 1.0 {
        return format!("{value:.2} V");
    }
    format!("{value:.3} V")
}

fn paint_time_labels(painter: &egui::Painter, plot: egui::Rect, duration: f64) {
    let palette = crate::design::palette_for(painter.ctx());
    for step in 0..=4 {
        let fraction = f64::from(step) / 4.0;
        let x = plot.left() + plot.width() * step as f32 / 4.0;
        painter.text(
            egui::pos2(x, plot.bottom() + 4.0),
            egui::Align2::CENTER_TOP,
            format_time(duration * fraction),
            crate::design::mono(10.0),
            palette.text_muted,
        );
    }
}

fn format_time(seconds: f64) -> String {
    if seconds >= 1.0 {
        return format!("{seconds:.2} s");
    }
    format!("{:.3} ms", seconds * 1e3)
}

fn paint_series(
    painter: &egui::Painter,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    node: usize,
    range: (f64, f64),
    color: egui::Color32,
) {
    let points = series_points(plot, trace, node, range);
    if points.len() < 2 {
        return;
    }
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, color)));
}

fn series_points(
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    node: usize,
    range: (f64, f64),
) -> Vec<egui::Pos2> {
    let step = sample_step(trace.times.len(), plot.width());
    let mut points = Vec::new();
    for index in (0..trace.times.len()).step_by(step) {
        push_point(&mut points, plot, trace, node, range, index);
    }
    push_last_point(&mut points, plot, trace, node, range);
    points
}

fn sample_step(count: usize, width: f32) -> usize {
    (count / width.max(1.0) as usize).max(1)
}

fn push_point(
    points: &mut Vec<egui::Pos2>,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    node: usize,
    range: (f64, f64),
    index: usize,
) {
    let value = trace.values[index][node];
    if !value.is_finite() {
        return;
    }
    points.push(egui::pos2(
        time_to_x(plot, trace.duration(), trace.times[index]),
        value_to_y(plot, range, value),
    ));
}

fn push_last_point(
    points: &mut Vec<egui::Pos2>,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    node: usize,
    range: (f64, f64),
) {
    let Some(index) = trace.times.len().checked_sub(1) else {
        return;
    };
    push_point(points, plot, trace, node, range, index);
}

fn time_to_x(plot: egui::Rect, duration: f64, time: f64) -> f32 {
    if duration <= 0.0 {
        return plot.left();
    }
    plot.left() + plot.width() * (time / duration) as f32
}

fn value_to_y(plot: egui::Rect, range: (f64, f64), value: f64) -> f32 {
    let span = (range.1 - range.0).max(1e-12);
    plot.bottom() - plot.height() * ((value - range.0) / span) as f32
}

fn paint_legend(
    painter: &egui::Painter,
    plot: egui::Rect,
    trace: &crate::transient::TransientTrace,
    selected: &BTreeSet<usize>,
) {
    let palette = crate::design::palette_for(painter.ctx());
    let mut y = plot.top() + 6.0;
    for (order, &node) in selected.iter().take(crate::probe::MAX_SERIES).enumerate() {
        let color = SERIES_COLORS[order % SERIES_COLORS.len()];
        paint_legend_entry(painter, plot, y, color, &trace.nodes[node], palette);
        y += 12.0;
    }
}

fn paint_legend_entry(
    painter: &egui::Painter,
    plot: egui::Rect,
    y: f32,
    color: egui::Color32,
    name: &str,
    palette: crate::design::Palette,
) {
    let swatch =
        egui::Rect::from_min_size(egui::pos2(plot.right() - 130.0, y), egui::vec2(8.0, 8.0));
    painter.rect_filled(swatch, 1.0, color);
    painter.text(
        egui::pos2(plot.right() - 118.0, y),
        egui::Align2::LEFT_TOP,
        name,
        crate::design::mono(10.0),
        palette.text,
    );
}
