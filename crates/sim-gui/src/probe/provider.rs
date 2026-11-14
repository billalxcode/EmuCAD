use super::*;

/// Adapter that exposes the current PB5 sample through the read-only API.
#[derive(Clone, Copy, Debug)]
pub struct SampleProvider {
    pub sample: Sample,
}

impl AnalysisProvider for SampleProvider {
    fn pin_voltage(&self, _component: ComponentId, pin: &str, cycle: u64) -> Option<f64> {
        if cycle != self.sample.cycle || pin != "PB5" {
            return None;
        }
        Some(f64::from(self.sample.pb5_millivolts) / 1000.0)
    }

    fn node_voltage(&self, _wire: WireId, _cycle: u64) -> Option<f64> {
        None
    }

    fn branch_current(&self, _component: ComponentId, pin: &str, cycle: u64) -> Option<f64> {
        if cycle != self.sample.cycle || pin != "LED" {
            return None;
        }
        Some(f64::from(self.sample.led_microamps) / 1_000_000.0)
    }
}

pub(super) fn source_value(
    provider: &dyn AnalysisProvider,
    source: ProbeSource,
    cycle: u64,
) -> Option<f64> {
    match source {
        ProbeSource::Pin { component, pin } => provider.pin_voltage(component, pin, cycle),
        ProbeSource::Net { wire } => provider.node_voltage(wire, cycle),
        ProbeSource::Branch { component, pin } => provider.branch_current(component, pin, cycle),
    }
}
