//! Read-only probe model, bounded sample buffers, and decimation helpers.

use crate::model::{ComponentId, WireId};
use crate::sim::Sample;
use eframe::egui;
use std::collections::VecDeque;

/// Maximum number of analog series in one plot.
pub const MAX_SERIES: usize = 8;
/// Default probe sampling interval in CPU cycles.
pub const DEFAULT_SAMPLE_EVERY: u64 = 100;
const BUFFER_SECONDS: f64 = 10.0;
const CLOCK_HZ: f64 = 16_000_000.0;
const MAX_SAMPLES: usize = 100_000;

/// Read-only source selected for a probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeSource {
    Pin {
        component: ComponentId,
        pin: &'static str,
    },
    /// Reserved for solver-backed net probes.
    #[allow(dead_code)]
    Net { wire: WireId },
    /// Branch current probe.
    Branch {
        component: ComponentId,
        pin: &'static str,
    },
}

/// Per-probe sampling interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampling {
    pub every_cycles: u64,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            every_cycles: DEFAULT_SAMPLE_EVERY,
        }
    }
}

impl Sampling {
    /// Returns the bounded buffer capacity for a 10 second window.
    #[must_use]
    pub fn capacity(self) -> usize {
        let every = self.every_cycles.max(1) as f64;
        let total = (BUFFER_SECONDS * CLOCK_HZ / every) as usize;
        total.clamp(2, MAX_SAMPLES)
    }
}

/// Probe identity and rendering metadata.
#[derive(Clone, Debug)]
pub struct ProbeSpec {
    pub label: String,
    pub unit: &'static str,
    pub color: egui::Color32,
    pub source: ProbeSource,
    pub sampling: Sampling,
}

/// One sampled value on the simulation timeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbePoint {
    pub cycle: u64,
    pub value: f64,
}

mod buffer;
mod decimate;
mod provider;
mod set;
#[cfg(test)]
mod tests;
mod view;

pub use buffer::{AnalysisProvider, ProbeBuffer};
pub use decimate::decimate;
pub use provider::SampleProvider;
use provider::source_value;
pub use set::ProbeSet;
pub use view::{PlotView, value_range_for, window};
