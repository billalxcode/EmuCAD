use super::*;

pub(crate) fn show_analog_graph(
    ui: &mut egui::Ui,
    probes: &ProbeSet,
    view: &mut PlotView,
    status: Status,
) {
    let window = probe::window(probes.span(), *view);
    let ranges = analog_ranges(probes, view, window);
    ui.label(analog_header(probes, status, window));
    plot_controls(ui, view);
    let width = ui.available_width().max(320.0);
    let (response, painter) =
        ui.allocate_painter(egui::vec2(width, 190.0), egui::Sense::click_and_drag());
    let plot = response.rect.shrink2(egui::vec2(42.0, 14.0));
    handle_plot_input(&response, view, plot, window);
    paint_analog_plot(&painter, response.rect, plot, probes, window, ranges, *view);
}

type PlotRanges = (Option<(f64, f64)>, Option<(f64, f64)>);

pub(crate) fn analog_ranges(probes: &ProbeSet, view: &PlotView, window: (u64, u64)) -> PlotRanges {
    if !view.autoscale {
        return (Some((0.0, 5.0)), Some((0.0, 0.05)));
    }
    (
        probe::value_range_for(probes.probes(), window, "V"),
        probe::value_range_for(probes.probes(), window, "A"),
    )
}

pub(crate) fn analog_header(probes: &ProbeSet, status: Status, window: (u64, u64)) -> String {
    let samples = probe_sample_count(probes);
    let last = probe_last_text(probes);
    format!(
        "{} series  {} samples  last {}  cycle {}  window {}..{}",
        probes.probes().len(),
        samples,
        last,
        status.cycles,
        window.0,
        window.1
    )
}

pub(crate) fn probe_sample_count(probes: &ProbeSet) -> usize {
    probes
        .probes()
        .iter()
        .map(|probe| probe.samples().len())
        .sum()
}

pub(crate) fn probe_last_text(probes: &ProbeSet) -> String {
    let Some(probe) = probes.probes().first() else {
        return "n/a".to_string();
    };
    probe.latest().map_or_else(
        || "n/a".to_string(),
        |point| format!("{:.2}{}", point.value, probe.spec().unit),
    )
}

pub(crate) fn plot_controls(ui: &mut egui::Ui, view: &mut PlotView) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut view.autoscale, "Auto");
        ui.checkbox(&mut view.legend, "Legend");
        if ui.button("-").clicked() {
            view.zoom = (view.zoom * 0.8).clamp(0.25, 8.0);
        }
        if ui.button("+").clicked() {
            view.zoom = (view.zoom * 1.25).clamp(0.25, 8.0);
        }
        if ui.button("Reset").clicked() {
            *view = PlotView::default();
        }
        ui.label(format!("zoom {:.1}x", view.zoom));
    });
}

pub(crate) fn handle_plot_input(
    response: &egui::Response,
    view: &mut PlotView,
    plot: egui::Rect,
    window: (u64, u64),
) {
    if response.dragged() {
        let fraction = f64::from(response.drag_delta().x / plot.width().max(1.0));
        view.pan = (view.pan - fraction).clamp(-0.95, 0.95);
    }
    let scroll = response.ctx.input(|input| input.smooth_scroll_delta.y);
    if response.hovered() && scroll != 0.0 {
        view.zoom = (view.zoom * (1.0 + scroll * 0.001)).clamp(0.25, 8.0);
    }
    view.cursor = response
        .hover_pos()
        .filter(|pos| plot.contains(*pos))
        .map(|pos| cycle_at(plot, window, pos.x));
}

pub(crate) fn paint_analog_plot(
    painter: &egui::Painter,
    rect: egui::Rect,
    plot: egui::Rect,
    probes: &ProbeSet,
    window: (u64, u64),
    ranges: PlotRanges,
    view: PlotView,
) {
    painter.rect_filled(
        rect,
        4.0,
        crate::design::palette_for(painter.ctx()).graph_bg,
    );
    paint_value_grid(painter, plot, ranges);
    for probe in probes.probes() {
        paint_probe_series(painter, plot, probe, window, ranges);
    }
    if view.legend {
        paint_analog_legend(painter, plot, probes);
    }
    paint_analog_cursor(painter, plot, probes, window, view.cursor);
}

pub(crate) fn paint_value_grid(painter: &egui::Painter, plot: egui::Rect, ranges: PlotRanges) {
    let stroke = egui::Stroke::new(1.0, crate::design::palette_for(painter.ctx()).graph_grid);
    for step in 0..=4 {
        let y = plot.top() + plot.height() * step as f32 / 4.0;
        painter.line_segment(
            [egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
            stroke,
        );
    }
    if let Some(range) = ranges.0 {
        paint_axis_labels(painter, plot, range, false);
    }
    if let Some(range) = ranges.1 {
        paint_axis_labels(painter, plot, range, true);
    }
}

pub(crate) fn paint_axis_labels(
    painter: &egui::Painter,
    plot: egui::Rect,
    range: (f64, f64),
    right: bool,
) {
    let colors = crate::design::palette_for(painter.ctx());
    let color = if right { LED_COLOR } else { colors.text_muted };
    for step in 0..=4 {
        paint_axis_label(painter, plot, range, right, step, color);
    }
}

pub(crate) fn paint_axis_label(
    painter: &egui::Painter,
    plot: egui::Rect,
    range: (f64, f64),
    right: bool,
    step: i32,
    color: egui::Color32,
) {
    let (x, align) = axis_anchor(plot, right);
    painter.text(
        egui::pos2(x, axis_y(plot, step)),
        align,
        axis_label(axis_value(range, step), range),
        crate::design::mono(10.0),
        color,
    );
}

pub(crate) fn axis_value(range: (f64, f64), step: i32) -> f64 {
    range.0 + (range.1 - range.0) * f64::from(step) / 4.0
}

pub(crate) fn axis_y(plot: egui::Rect, step: i32) -> f32 {
    plot.bottom() - plot.height() * step as f32 / 4.0
}

pub(crate) fn axis_anchor(plot: egui::Rect, right: bool) -> (f32, egui::Align2) {
    if right {
        return (plot.right() + 6.0, egui::Align2::LEFT_CENTER);
    }
    (plot.left() - 6.0, egui::Align2::RIGHT_CENTER)
}

pub(crate) fn axis_label(value: f64, range: (f64, f64)) -> String {
    if (range.1 - range.0).abs() < 1.0 {
        return format!("{value:.3}");
    }
    format!("{value:.1}")
}

pub(crate) fn paint_probe_series(
    painter: &egui::Painter,
    plot: egui::Rect,
    probe: &probe::ProbeBuffer,
    window: (u64, u64),
    ranges: PlotRanges,
) {
    let Some(range) = series_range(probe, ranges) else {
        return;
    };
    let screen = probe_screen_points(plot, probe, window, range);
    if screen.len() < 2 {
        return;
    }
    painter.add(egui::Shape::line(
        screen,
        egui::Stroke::new(1.5, probe.spec().color),
    ));
}

pub(crate) fn series_range(probe: &probe::ProbeBuffer, ranges: PlotRanges) -> Option<(f64, f64)> {
    if probe.spec().unit == "A" {
        return ranges.1;
    }
    ranges.0
}

pub(crate) fn probe_screen_points(
    plot: egui::Rect,
    probe: &probe::ProbeBuffer,
    window: (u64, u64),
    range: (f64, f64),
) -> Vec<egui::Pos2> {
    let limit = (plot.width() as usize).max(32);
    let points = probe::decimate(probe.samples(), limit);
    points
        .iter()
        .filter(|point| point.cycle >= window.0 && point.cycle <= window.1)
        .map(|point| {
            egui::pos2(
                cycle_to_x(plot, window, point.cycle),
                value_to_y(plot, range, point.value),
            )
        })
        .collect()
}

pub(crate) fn paint_analog_legend(painter: &egui::Painter, plot: egui::Rect, probes: &ProbeSet) {
    let mut y = plot.top() + 6.0;
    for probe in probes.probes().iter().take(probe::MAX_SERIES) {
        let color = probe.spec().color;
        let swatch =
            egui::Rect::from_min_size(egui::pos2(plot.right() - 150.0, y), egui::vec2(8.0, 8.0));
        painter.rect_filled(swatch, 1.0, color);
        painter.text(
            egui::pos2(plot.right() - 138.0, y),
            egui::Align2::LEFT_TOP,
            format!("{} ({})", probe.spec().label, probe.spec().unit),
            crate::design::mono(10.0),
            crate::design::palette_for(painter.ctx()).text,
        );
        y += 12.0;
    }
}

pub(crate) fn paint_analog_cursor(
    painter: &egui::Painter,
    plot: egui::Rect,
    probes: &ProbeSet,
    window: (u64, u64),
    cursor: Option<u64>,
) {
    let Some(cycle) = cursor else {
        return;
    };
    let x = cycle_to_x(plot, window, cycle);
    paint_cursor_line(painter, plot, x);
    paint_cursor_label(painter, plot, x, probes, cycle);
}

pub(crate) fn paint_cursor_line(painter: &egui::Painter, plot: egui::Rect, x: f32) {
    painter.line_segment(
        [egui::pos2(x, plot.top()), egui::pos2(x, plot.bottom())],
        egui::Stroke::new(1.0, crate::design::palette_for(painter.ctx()).warning),
    );
}

pub(crate) fn paint_cursor_label(
    painter: &egui::Painter,
    plot: egui::Rect,
    x: f32,
    probes: &ProbeSet,
    cycle: u64,
) {
    painter.text(
        egui::pos2(x + 4.0, plot.top() + 4.0),
        egui::Align2::LEFT_TOP,
        analog_cursor_text(probes, cycle),
        crate::design::mono(10.0),
        crate::design::palette_for(painter.ctx()).text,
    );
}

pub(crate) fn analog_cursor_text(probes: &ProbeSet, cycle: u64) -> String {
    let mut text = format!("cycle {cycle}");
    for probe in probes.probes().iter().take(4) {
        let Some(value) = probe_value_at(probe, cycle) else {
            continue;
        };
        text.push_str(&format!(
            "  {} {:.2}{}",
            probe.spec().label,
            value,
            probe.spec().unit
        ));
    }
    text
}

pub(crate) fn probe_value_at(probe: &probe::ProbeBuffer, cycle: u64) -> Option<f64> {
    probe
        .samples()
        .iter()
        .rev()
        .find(|point| point.cycle <= cycle)
        .map(|point| point.value)
}

pub(crate) fn cycle_to_x(plot: egui::Rect, window: (u64, u64), cycle: u64) -> f32 {
    let span = window.1.saturating_sub(window.0).max(1) as f32;
    let time = (cycle.saturating_sub(window.0)) as f32 / span;
    plot.left() + time.clamp(0.0, 1.0) * plot.width()
}

pub(crate) fn cycle_at(plot: egui::Rect, window: (u64, u64), x: f32) -> u64 {
    let time = ((x - plot.left()) / plot.width().max(1.0)).clamp(0.0, 1.0);
    window.0 + (time * (window.1 - window.0) as f32) as u64
}

pub(crate) fn value_to_y(plot: egui::Rect, range: (f64, f64), value: f64) -> f32 {
    let span = (range.1 - range.0).abs().max(f64::EPSILON);
    let level = ((value - range.0) / span) as f32;
    plot.bottom() - level.clamp(0.0, 1.0) * plot.height()
}
