//! Transient analysis and traced node voltages.

use crate::model::Document;
use crate::netlist::{self, Netlist};
use sim_analysis::{TransientConfig, TransientSample, transient as run_transient};
use sim_core::{NodeId, Solution};

const MAX_STEPS: usize = 100_000;
const MAX_SECONDS: f64 = 10.0;

/// User-facing transient settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TransientSettings {
    pub dt: f64,
    pub steps: usize,
    pub use_dc_op: bool,
}

impl Default for TransientSettings {
    fn default() -> Self {
        Self {
            dt: 1e-5,
            steps: 1000,
            use_dc_op: true,
        }
    }
}

/// Sampled node voltages over time.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TransientTrace {
    pub times: Vec<f64>,
    pub nodes: Vec<String>,
    pub values: Vec<Vec<f64>>,
    pub dt: f64,
}

impl TransientTrace {
    /// Returns the sample index nearest to the requested time.
    #[cfg(test)]
    #[must_use]
    pub fn index_at(&self, time: f64) -> Option<usize> {
        self.times
            .iter()
            .position(|sample| *sample >= time)
            .or_else(|| self.times.len().checked_sub(1))
    }

    /// Returns the final time in seconds.
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.times.last().copied().unwrap_or(0.0)
    }
}

/// Solves a transient analysis for a schematic document.
pub(crate) fn solve(
    document: &Document,
    settings: &TransientSettings,
) -> Result<TransientTrace, String> {
    validate(settings)?;
    let netlist = netlist::extract(document).map_err(|error| error.to_string())?;
    let circuit = netlist::build_circuit(&netlist)?;
    let config = transient_config(settings);
    let samples = run_transient(&circuit, &config).map_err(|error| error.to_string())?;
    Ok(trace(&netlist, &samples, settings.dt))
}

fn validate(settings: &TransientSettings) -> Result<(), String> {
    if !settings.dt.is_finite() || settings.dt <= 0.0 {
        return Err("transient dt must be positive".to_string());
    }
    if settings.steps == 0 || settings.steps > MAX_STEPS {
        return Err(format!("transient steps must be 1 to {MAX_STEPS}"));
    }
    if settings.dt * settings.steps as f64 > MAX_SECONDS {
        return Err(format!("transient duration exceeds {MAX_SECONDS} s"));
    }
    Ok(())
}

fn transient_config(settings: &TransientSettings) -> TransientConfig {
    TransientConfig {
        dt: settings.dt,
        steps: settings.steps,
        use_dc_op: settings.use_dc_op,
        ..TransientConfig::default()
    }
}

fn trace(netlist: &Netlist, samples: &[TransientSample], dt: f64) -> TransientTrace {
    let times = samples.iter().map(sample_time).collect();
    let values = samples
        .iter()
        .map(|sample| node_values(&sample.solution, netlist.nodes.len()))
        .collect();
    TransientTrace {
        times,
        nodes: netlist.nodes.iter().map(|node| node.name.clone()).collect(),
        values,
        dt,
    }
}

fn sample_time(sample: &TransientSample) -> f64 {
    sample.time.as_ps() as f64 * 1e-12
}

fn node_values(solution: &Solution, count: usize) -> Vec<f64> {
    (0..count)
        .map(|index| {
            let raw = u32::try_from(index).expect("node index fits u32");
            solution.voltage(NodeId::new(raw)).unwrap_or(f64::NAN)
        })
        .collect()
}

#[cfg(test)]
mod tests;
