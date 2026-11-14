use super::*;

/// Visible viewport controls for an analog plot.
#[derive(Clone, Copy, Debug)]
pub struct PlotView {
    pub zoom: f32,
    pub pan: f64,
    pub autoscale: bool,
    pub legend: bool,
    pub cursor: Option<u64>,
}

impl Default for PlotView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: 0.0,
            autoscale: true,
            legend: true,
            cursor: None,
        }
    }
}

/// Returns the visible cycle window after zoom and pan.
#[must_use]
pub fn window(span: (u64, u64), view: PlotView) -> (u64, u64) {
    let (start, end) = span;
    let width = end.saturating_sub(start).max(1);
    let zoom = f64::from(view.zoom.clamp(0.25, 8.0));
    let visible = ((width as f64 / zoom) as u64).max(1);
    let center = start as i64 + width as i64 / 2;
    let shift = (view.pan * visible as f64) as i64;
    let center = (center + shift).max(0) as u64;
    let begin = center.saturating_sub(visible / 2);
    (begin, begin + visible)
}

/// Returns the value range for probes with one unit.
#[must_use]
pub fn value_range_for(
    probes: &[ProbeBuffer],
    window: (u64, u64),
    unit: &str,
) -> Option<(f64, f64)> {
    range_for(probes, window, |probe| probe.spec().unit == unit)
}

fn range_for(
    probes: &[ProbeBuffer],
    window: (u64, u64),
    include: impl Fn(&ProbeBuffer) -> bool,
) -> Option<(f64, f64)> {
    probes
        .iter()
        .filter(|probe| include(probe))
        .filter_map(|probe| probe_range(probe, window))
        .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
        .map(|(min, max)| padded_range(min, max))
}

fn probe_range(probe: &ProbeBuffer, window: (u64, u64)) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found = false;
    for point in probe.samples() {
        if point.cycle < window.0 || point.cycle > window.1 {
            continue;
        }
        min = min.min(point.value);
        max = max.max(point.value);
        found = true;
    }
    found.then_some((min, max))
}

fn padded_range(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < f64::EPSILON {
        return (min - 0.5, max + 0.5);
    }
    (min, max)
}
