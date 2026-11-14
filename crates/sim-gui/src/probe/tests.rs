use super::{
    AnalysisProvider, MAX_SERIES, PlotView, ProbeBuffer, ProbePoint, ProbeSet, ProbeSource,
    ProbeSpec, Sampling, decimate, window,
};
use crate::model::{ComponentId, WireId};
use crate::sim::Sample;
use std::collections::VecDeque;

fn spec(every_cycles: u64) -> ProbeSpec {
    ProbeSpec {
        label: "PB5".to_string(),
        unit: "V",
        color: eframe::egui::Color32::WHITE,
        source: ProbeSource::Pin {
            component: ComponentId(1),
            pin: "PB5",
        },
        sampling: Sampling { every_cycles },
    }
}

fn point(cycle: u64, value: f64) -> ProbePoint {
    ProbePoint { cycle, value }
}

#[test]
fn buffer_drops_oldest_samples() {
    let mut buffer = ProbeBuffer::new(spec(10_000_000));
    for cycle in 0..20 {
        buffer.push(point(cycle, cycle as f64));
    }
    assert_eq!(buffer.samples().len(), 16);
    assert_eq!(buffer.latest().expect("latest").cycle, 19);
}

#[test]
fn decimation_preserves_extremes() {
    let samples: VecDeque<_> = (0..100).map(|cycle| point(cycle, cycle as f64)).collect();
    let output = decimate(&samples, 10);
    assert!(output.len() <= 20);
    assert!(output.iter().any(|point| point.value == 0.0));
    assert!(output.iter().any(|point| point.value == 99.0));
}

#[test]
fn probe_set_limits_series() {
    let mut set = ProbeSet::default();
    for _ in 0..MAX_SERIES {
        assert!(set.add(spec(100)));
    }
    assert!(!set.add(spec(100)));
}

struct CycleProvider;

impl AnalysisProvider for CycleProvider {
    fn pin_voltage(&self, _component: ComponentId, _pin: &str, cycle: u64) -> Option<f64> {
        Some(cycle as f64)
    }
    fn node_voltage(&self, _wire: WireId, _cycle: u64) -> Option<f64> {
        None
    }
    fn branch_current(&self, _component: ComponentId, _pin: &str, _cycle: u64) -> Option<f64> {
        None
    }
}

#[test]
fn probe_set_samples_from_provider() {
    let mut set = ProbeSet::default();
    assert!(set.add(spec(100)));
    set.sample_from(&CycleProvider, 100);
    assert_eq!(set.probes()[0].latest(), Some(point(100, 100.0)));
}

#[test]
fn probe_set_filters_per_probe_sampling() {
    let mut set = ProbeSet::default();
    assert!(set.add(spec(10)));
    assert!(set.add(spec(20)));
    set.sample_from(&CycleProvider, 10);
    set.sample_from(&CycleProvider, 20);
    assert_eq!(set.probes()[0].samples().len(), 2);
    assert_eq!(set.probes()[1].samples().len(), 1);
}

#[test]
fn sample_provider_exposes_pb5_voltage() {
    let sample = Sample {
        cycle: 7,
        pb5_high: true,
        pb5_millivolts: 4200,
        led_on: true,
        led_microamps: 12_000,
    };
    let provider = super::SampleProvider { sample };
    assert_eq!(provider.pin_voltage(ComponentId(1), "PB5", 7), Some(4.2));
    assert_eq!(provider.pin_voltage(ComponentId(1), "PB5", 8), None);
    assert_eq!(
        provider.branch_current(ComponentId(1), "LED", 7),
        Some(0.012)
    );
}

#[test]
fn plot_window_respects_zoom_and_pan() {
    assert_eq!(window((0, 100), PlotView::default()), (0, 100));
    let zoomed = window(
        (0, 100),
        PlotView {
            zoom: 2.0,
            ..PlotView::default()
        },
    );
    assert_eq!(zoomed, (25, 75));
    let panned = window(
        (0, 100),
        PlotView {
            zoom: 2.0,
            pan: 0.2,
            ..PlotView::default()
        },
    );
    assert!(panned.0 > zoomed.0);
}

#[test]
fn value_range_tracks_visible_samples() {
    let mut buffer = ProbeBuffer::new(spec(100));
    buffer.push(point(10, 1.0));
    buffer.push(point(20, 3.0));
    buffer.push(point(30, 2.0));
    assert_eq!(
        super::value_range_for(std::slice::from_ref(&buffer), (0, 25), "V"),
        Some((1.0, 3.0))
    );
    assert_eq!(
        super::value_range_for(std::slice::from_ref(&buffer), (25, 40), "V"),
        Some((1.5, 2.5))
    );
}
