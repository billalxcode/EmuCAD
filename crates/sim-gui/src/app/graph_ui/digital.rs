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
    let colors = crate::design::palette_for(painter.ctx());
    painter.rect_filled(rect, 4.0, colors.graph_bg);
    let middle = plot.center().y;
    let stroke = egui::Stroke::new(1.0, colors.graph_grid);
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
    let stroke = egui::Stroke::new(1.0, crate::design::palette_for(painter.ctx()).warning);
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
    let dim = crate::design::palette_for(painter.ctx()).text_muted;
    let font = crate::design::mono(10.0);
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
