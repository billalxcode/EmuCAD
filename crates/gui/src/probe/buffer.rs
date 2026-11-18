use super::*;

/// Bounded probe sample history.
#[derive(Clone, Debug)]
pub struct ProbeBuffer {
    pub(super) spec: ProbeSpec,
    samples: VecDeque<ProbePoint>,
    capacity: usize,
}

impl ProbeBuffer {
    /// Creates an empty buffer with the capacity derived from its sampling.
    #[must_use]
    pub fn new(spec: ProbeSpec) -> Self {
        let capacity = spec.sampling.capacity();
        Self {
            spec,
            samples: VecDeque::new(),
            capacity,
        }
    }

    /// Appends a point, dropping the oldest beyond the capacity.
    pub fn push(&mut self, point: ProbePoint) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(point);
    }

    /// Removes all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Returns probe metadata.
    #[must_use]
    pub fn spec(&self) -> &ProbeSpec {
        &self.spec
    }

    /// Returns samples in time order.
    #[must_use]
    pub fn samples(&self) -> &VecDeque<ProbePoint> {
        &self.samples
    }

    /// Returns the newest sample.
    #[must_use]
    pub fn latest(&self) -> Option<ProbePoint> {
        self.samples.back().copied()
    }
}

/// Read-only provider used by analysis plugins and the GUI adapter.
pub trait AnalysisProvider {
    /// Returns a pin voltage in volts.
    fn pin_voltage(&self, component: ComponentId, pin: &str, cycle: u64) -> Option<f64>;
    /// Returns a net voltage in volts.
    fn node_voltage(&self, wire: WireId, cycle: u64) -> Option<f64>;
    /// Returns a branch current in amperes.
    fn branch_current(&self, component: ComponentId, pin: &str, cycle: u64) -> Option<f64>;
}
