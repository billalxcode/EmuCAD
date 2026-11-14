use super::*;

/// Set of at most eight probes.
#[derive(Clone, Debug, Default)]
pub struct ProbeSet {
    probes: Vec<ProbeBuffer>,
}

impl ProbeSet {
    /// Adds a probe when the series limit is not reached.
    pub fn add(&mut self, spec: ProbeSpec) -> bool {
        if self.probes.len() >= MAX_SERIES {
            return false;
        }
        self.probes.push(ProbeBuffer::new(spec));
        true
    }

    /// Returns all probes.
    #[must_use]
    pub fn probes(&self) -> &[ProbeBuffer] {
        &self.probes
    }

    /// Returns true when a source already has a probe.
    #[must_use]
    pub fn contains(&self, source: ProbeSource) -> bool {
        self.probes.iter().any(|probe| probe.spec.source == source)
    }

    /// Clears sample history but keeps probe definitions.
    pub fn clear_samples(&mut self) {
        for probe in &mut self.probes {
            probe.clear();
        }
    }

    /// Appends one sample to every probe from a read-only provider.
    pub fn sample_from(&mut self, provider: &dyn AnalysisProvider, cycle: u64) {
        for probe in &mut self.probes {
            let every = probe.spec.sampling.every_cycles.max(1);
            if cycle % every != 0 {
                continue;
            }
            let Some(value) = source_value(provider, probe.spec.source, cycle) else {
                continue;
            };
            probe.push(ProbePoint { cycle, value });
        }
    }

    /// Returns the full cycle span across all probes.
    #[must_use]
    pub fn span(&self) -> (u64, u64) {
        let start = self
            .probes
            .iter()
            .filter_map(|probe| probe.samples().front().map(|point| point.cycle))
            .min()
            .unwrap_or(0);
        let end = self
            .probes
            .iter()
            .filter_map(|probe| probe.samples().back().map(|point| point.cycle))
            .max()
            .unwrap_or(start);
        if end <= start {
            (start, start + 1)
        } else {
            (start, end)
        }
    }
}
