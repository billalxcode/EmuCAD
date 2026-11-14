use super::*;

pub(crate) fn show_digital_graph(ui: &mut egui::Ui, waveform: &Waveform, status: Status) {
    let span = waveform.span(status.cycles);
    ui.label(graph_header(status, span));
    let width = ui.available_width().max(220.0);
    let (response, painter) = ui.allocate_painter(egui::vec2(width, 130.0), egui::Sense::hover());
    let plot = response.rect.shrink2(egui::vec2(38.0, 12.0));
    let cursor = response.hover_pos().filter(|pos| plot.contains(*pos));
    paint_digital_graph(&painter, response.rect, plot, waveform, span, cursor);
    ui.label(graph_cursor_text(waveform, plot, span, cursor));
}

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
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 22, 26));
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
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 46, 56));
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
    let color = if right {
        LED_COLOR
    } else {
        egui::Color32::from_rgb(150, 160, 175)
    };
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
        egui::FontId::monospace(9.0),
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
            egui::FontId::monospace(9.0),
            egui::Color32::from_rgb(190, 198, 210),
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
        egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 215, 225)),
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
        egui::FontId::monospace(9.0),
        egui::Color32::from_rgb(210, 218, 228),
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

pub(crate) fn graph_header(status: Status, span: (u64, u64)) -> String {
    format!(
        "PB5 {}  LED {}  cycles {} to {}",
        high_low(status.pb5_high),
        on_off(status.led_on),
        span.0,
        span.1
    )
}

pub(crate) fn graph_cursor_text(
    waveform: &Waveform,
    plot: egui::Rect,
    span: (u64, u64),
    cursor: Option<egui::Pos2>,
) -> String {
    let Some(position) = cursor else {
        return "Hover the graph to inspect a cycle".to_string();
    };
    let cycle = cycle_at_x(plot, span, position.x);
    let (pb5, led) = waveform.state_at(cycle);
    format!("cycle {cycle}: PB5 {}  LED {}", high_low(pb5), on_off(led))
}

pub(crate) fn high_low(value: bool) -> &'static str {
    if value { "high" } else { "low" }
}

pub(crate) fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

pub(crate) fn paint_digital_graph(
    painter: &egui::Painter,
    rect: egui::Rect,
    plot: egui::Rect,
    waveform: &Waveform,
    span: (u64, u64),
    cursor: Option<egui::Pos2>,
) {
    paint_graph_background(painter, rect, plot);
    paint_signals(painter, plot, waveform.samples(), span);
    paint_signal_labels(painter, rect, plot);
    paint_cursor(painter, plot, cursor);
}

pub(crate) fn paint_signals(
    painter: &egui::Painter,
    plot: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
) {
    let (top, bottom) = signal_rows(plot);
    paint_signal(painter, top, samples, span, pb5_value, PB5_COLOR);
    paint_signal(painter, bottom, samples, span, led_value, LED_COLOR);
}

pub(crate) fn signal_rows(plot: egui::Rect) -> (egui::Rect, egui::Rect) {
    let middle = plot.center().y;
    (
        egui::Rect::from_min_max(plot.min, egui::pos2(plot.max.x, middle)),
        egui::Rect::from_min_max(egui::pos2(plot.min.x, middle), plot.max),
    )
}

pub(crate) fn paint_graph_background(painter: &egui::Painter, rect: egui::Rect, plot: egui::Rect) {
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 22, 26));
    let middle = plot.center().y;
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 78, 90));
    painter.line_segment(
        [
            egui::pos2(plot.left(), middle),
            egui::pos2(plot.right(), middle),
        ],
        stroke,
    );
}

pub(crate) fn paint_cursor(painter: &egui::Painter, plot: egui::Rect, cursor: Option<egui::Pos2>) {
    let Some(position) = cursor else {
        return;
    };
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 220, 120));
    painter.line_segment(
        [
            egui::pos2(position.x, plot.top()),
            egui::pos2(position.x, plot.bottom()),
        ],
        stroke,
    );
}

pub(crate) fn pb5_value(sample: &Sample) -> bool {
    sample.pb5_high
}

pub(crate) fn led_value(sample: &Sample) -> bool {
    sample.led_on
}

pub(crate) fn paint_signal(
    painter: &egui::Painter,
    row: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
    value: fn(&Sample) -> bool,
    color: egui::Color32,
) {
    let points = signal_points(row, samples, span, value);
    if points.is_empty() {
        return;
    }
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, color)));
}

pub(crate) fn paint_signal_labels(painter: &egui::Painter, rect: egui::Rect, plot: egui::Rect) {
    let dim = egui::Color32::from_rgb(150, 160, 170);
    let font = egui::FontId::monospace(10.0);
    let high = egui::pos2(rect.left() + 4.0, plot.top() + 4.0);
    let low = egui::pos2(rect.left() + 4.0, plot.bottom() - 4.0);
    painter.text(high, egui::Align2::LEFT_TOP, "PB5", font.clone(), dim);
    painter.text(low, egui::Align2::LEFT_BOTTOM, "LED", font, dim);
}

pub(crate) fn cycle_at_x(plot: egui::Rect, span: (u64, u64), x: f32) -> u64 {
    let ratio = ((x - plot.left()) / plot.width().max(1.0)).clamp(0.0, 1.0);
    let length = span.1.saturating_sub(span.0);
    span.0 + (f64::from(ratio) * length as f64).round() as u64
}

pub(crate) fn signal_points(
    row: egui::Rect,
    samples: &VecDeque<Sample>,
    span: (u64, u64),
    value: fn(&Sample) -> bool,
) -> Vec<egui::Pos2> {
    let Some(first) = samples.front() else {
        return Vec::new();
    };
    let map = SignalMap::new(row, span);
    let mut points = vec![egui::pos2(row.left(), map.y(value(first)))];
    let mut previous = first;
    for sample in samples.iter().skip(1) {
        map.transition(&mut points, previous, sample, value);
        previous = sample;
    }
    points.push(egui::pos2(row.right(), map.y(value(previous))));
    points
}

struct SignalMap {
    row: egui::Rect,
    start: u64,
    width: f32,
}

impl SignalMap {
    fn new(row: egui::Rect, span: (u64, u64)) -> Self {
        Self {
            row,
            start: span.0,
            width: span.1.saturating_sub(span.0).max(1) as f32,
        }
    }

    fn x(&self, cycle: u64) -> f32 {
        self.row.left() + (cycle.saturating_sub(self.start) as f32 / self.width) * self.row.width()
    }

    fn y(&self, on: bool) -> f32 {
        if on {
            self.row.top()
        } else {
            self.row.bottom()
        }
    }

    fn transition(
        &self,
        points: &mut Vec<egui::Pos2>,
        previous: &Sample,
        sample: &Sample,
        value: fn(&Sample) -> bool,
    ) {
        let x = self.x(sample.cycle);
        points.push(egui::pos2(x, self.y(value(previous))));
        points.push(egui::pos2(x, self.y(value(sample))));
    }
}

pub(crate) fn world_to_grid(position: egui::Pos2) -> GridPos {
    GridPos::new(position.x.round() as i32, position.y.round() as i32)
}

pub(crate) fn wire_color_message(has_color: bool) -> String {
    if has_color {
        return "Wire color updated".to_string();
    }
    "Wire color reset".to_string()
}

pub(crate) fn icon_button(icon: &str, label: &str) -> egui::Button<'static> {
    egui::Button::new(format!("{icon} {label}"))
}

pub(crate) fn pb5_probe_spec() -> ProbeSpec {
    ProbeSpec {
        label: "PB5 voltage".to_string(),
        unit: "V",
        color: PB5_COLOR,
        source: ProbeSource::Pin {
            component: ComponentId(0),
            pin: "PB5",
        },
        sampling: Sampling::default(),
    }
}

pub(crate) fn led_probe_spec() -> ProbeSpec {
    ProbeSpec {
        label: "LED current".to_string(),
        unit: "A",
        color: LED_COLOR,
        source: ProbeSource::Branch {
            component: ComponentId(0),
            pin: "LED",
        },
        sampling: Sampling::default(),
    }
}

pub(crate) fn sample_combo(ui: &mut egui::Ui, current: u64) -> u64 {
    let mut every = current;
    egui::ComboBox::from_id_salt("sample_every")
        .selected_text(format!("N {every}"))
        .show_ui(ui, |ui| {
            for value in [10, 100, 1000] {
                ui.selectable_value(&mut every, value, format!("N {value}"));
            }
        });
    every
}

pub(crate) fn run_button_label(running: bool) -> egui::Button<'static> {
    if running {
        icon_button("||", "Pause")
    } else {
        icon_button(">", "Run")
    }
}

pub(crate) fn nudge_delta(ui: &egui::Ui) -> (i32, i32) {
    let step = if ui.input(|input| input.modifiers.shift) {
        5
    } else {
        1
    };
    let mut delta = (0, 0);
    ui.input(|input| {
        delta.0 = axis_delta(input, egui::Key::ArrowLeft, egui::Key::ArrowRight, step);
        delta.1 = axis_delta(input, egui::Key::ArrowUp, egui::Key::ArrowDown, step);
    });
    delta
}

pub(crate) fn axis_delta(
    input: &egui::InputState,
    negative: egui::Key,
    positive: egui::Key,
    step: i32,
) -> i32 {
    let mut delta = 0;
    if input.key_pressed(negative) {
        delta -= step;
    }
    if input.key_pressed(positive) {
        delta += step;
    }
    delta
}

pub(crate) fn flow_phase(enabled: bool, time: f64) -> Option<f32> {
    if !enabled {
        return None;
    }
    Some((time * 24.0).rem_euclid(1000.0) as f32)
}

pub(crate) fn grid_pos_f(position: GridPos) -> egui::Pos2 {
    egui::pos2(position.x as f32, position.y as f32)
}

pub(crate) fn content_bounds(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
) -> Option<(GridPos, GridPos)> {
    let mut min = None;
    let mut max = None;
    include_components(document, definitions, selection, &mut min, &mut max);
    include_wires(document, selection, &mut min, &mut max);
    Some((min?, max?))
}

pub(crate) fn include_components(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for instance in document.components() {
        if selection.is_some_and(|selection| !selection.contains_component(instance.id)) {
            continue;
        }
        let Some(bounds) = instance_bounds(definitions, instance) else {
            continue;
        };
        crate::model::extend_bounds(min, max, bounds.min);
        crate::model::extend_bounds(min, max, GridPos::new(bounds.max_x(), bounds.max_y()));
    }
}

pub(crate) fn instance_bounds(
    definitions: &[ComponentDefinition],
    instance: &ComponentInstance,
) -> Option<GridRect> {
    let definition = definitions
        .iter()
        .find(|definition| definition.id == instance.definition)?;
    Some(instance.bounds(definition))
}

pub(crate) fn include_wires(
    document: &Document,
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for wire in document.wires() {
        if selection.is_some_and(|selection| !selection.contains_wire(wire.id)) {
            continue;
        }
        for point in &wire.points {
            crate::model::extend_bounds(min, max, *point);
        }
    }
}

pub(crate) fn move_segment(points: &mut [GridPos], index: usize, target: GridPos) -> bool {
    let Some(&end) = points.get(index + 1) else {
        return false;
    };
    let start = points[index];
    if start.y == end.y {
        points[index].y = target.y;
        points[index + 1].y = target.y;
    } else {
        points[index].x = target.x;
        points[index + 1].x = target.x;
    }
    true
}

pub(crate) fn l_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target {
        return Vec::new();
    }
    if last.x == target.x || last.y == target.y {
        return vec![target];
    }
    let corner = if horizontal_first {
        GridPos::new(target.x, last.y)
    } else {
        GridPos::new(last.x, target.y)
    };
    vec![corner, target]
}

pub(crate) fn z_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target || last.x == target.x || last.y == target.y {
        return l_route(last, target, horizontal_first);
    }
    let path = if horizontal_first {
        let middle = GridPos::new((last.x + target.x) / 2, last.y);
        vec![middle, GridPos::new(middle.x, target.y), target]
    } else {
        let middle = GridPos::new(last.x, (last.y + target.y) / 2);
        vec![middle, GridPos::new(target.x, middle.y), target]
    };
    without_repeats(last, path)
}

pub(crate) fn without_repeats(start: GridPos, path: Vec<GridPos>) -> Vec<GridPos> {
    let mut result = Vec::new();
    let mut previous = start;
    for point in path {
        if point == previous {
            continue;
        }
        result.push(point);
        previous = point;
    }
    result
}
