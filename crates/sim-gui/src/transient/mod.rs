//! Transient analysis and traced node voltages.

use crate::model::Document;
use crate::netlist::{self, Netlist};
use serde::{Deserialize, Serialize};
use sim_analysis::{
    AdaptiveConfig, TransientConfig, TransientSample, transient as run_transient,
    transient_adaptive,
};
use sim_core::{NodeId, Solution};

const MAX_STEPS: usize = 100_000;
const MAX_SECONDS: f64 = 10.0;

/// User-facing transient settings.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct TransientSettings {
    pub dt: f64,
    pub steps: usize,
    pub gmin: f64,
    pub use_dc_op: bool,
    pub adaptive: bool,
    pub initial_dt: f64,
    pub min_dt: f64,
    pub max_dt: f64,
    pub tolerance: f64,
    pub max_samples: usize,
}

impl Default for TransientSettings {
    fn default() -> Self {
        Self {
            dt: 1e-5,
            steps: 1000,
            gmin: 1e-12,
            use_dc_op: true,
            adaptive: false,
            initial_dt: 1e-6,
            min_dt: 1e-12,
            max_dt: 1e-3,
            tolerance: 1e-3,
            max_samples: 100_000,
        }
    }
}

/// One device that can expose a terminal current probe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransientDevice {
    pub label: String,
    pub part: &'static str,
}

/// Metadata needed to build an incremental transient trace.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TraceMetadata {
    pub nodes: Vec<String>,
    pub devices: Vec<TransientDevice>,
    pub dt: f64,
    pub adaptive: bool,
}

/// Sampled node voltages and device currents over time.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TransientTrace {
    pub times: Vec<f64>,
    pub nodes: Vec<String>,
    pub values: Vec<Vec<f64>>,
    pub devices: Vec<TransientDevice>,
    pub currents: Vec<Vec<Option<f64>>>,
    pub dt: f64,
    pub adaptive: bool,
}

impl TransientTrace {
    /// Creates an empty trace with the given metadata.
    #[must_use]
    pub fn empty(metadata: TraceMetadata) -> Self {
        Self {
            times: Vec::new(),
            nodes: metadata.nodes,
            values: Vec::new(),
            devices: metadata.devices,
            currents: Vec::new(),
            dt: metadata.dt,
            adaptive: metadata.adaptive,
        }
    }

    /// Appends one sample.
    pub fn push(&mut self, sample: &TransientSample) {
        self.times.push(sample_time(sample));
        self.values
            .push(node_values(&sample.solution, self.nodes.len()));
        self.currents.push(sample.device_currents.clone());
    }

    /// Removes all samples while keeping metadata.
    pub fn clear_samples(&mut self) {
        self.times.clear();
        self.values.clear();
        self.currents.clear();
    }

    /// Returns the sample index nearest to the requested time.
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
    let samples = samples(&circuit, settings)?;
    let trace = trace(&netlist, &samples, settings);
    if trace.duration() > MAX_SECONDS {
        return Err(format!("transient duration exceeds {MAX_SECONDS} s"));
    }
    Ok(trace)
}

fn samples(
    circuit: &sim_core::Circuit,
    settings: &TransientSettings,
) -> Result<Vec<TransientSample>, String> {
    if settings.adaptive {
        return adaptive_samples(circuit, settings);
    }
    fixed_samples(circuit, settings)
}

fn fixed_samples(
    circuit: &sim_core::Circuit,
    settings: &TransientSettings,
) -> Result<Vec<TransientSample>, String> {
    let config = TransientConfig {
        dt: settings.dt,
        steps: settings.steps,
        gmin: settings.gmin,
        use_dc_op: settings.use_dc_op,
        ..TransientConfig::default()
    };
    run_transient(circuit, &config).map_err(|error| error.to_string())
}

fn adaptive_samples(
    circuit: &sim_core::Circuit,
    settings: &TransientSettings,
) -> Result<Vec<TransientSample>, String> {
    let base = TransientConfig {
        dt: settings.initial_dt,
        steps: settings.max_samples,
        gmin: settings.gmin,
        use_dc_op: settings.use_dc_op,
        ..TransientConfig::default()
    };
    let config = AdaptiveConfig {
        initial_dt: settings.initial_dt,
        min_dt: settings.min_dt,
        max_dt: settings.max_dt,
        tolerance: settings.tolerance,
        steps: settings.max_samples,
    };
    transient_adaptive(circuit, &base, &config).map_err(|error| error.to_string())
}

fn validate(settings: &TransientSettings) -> Result<(), String> {
    if !settings.gmin.is_finite() || settings.gmin < 0.0 {
        return Err("transient gmin must be non-negative".to_string());
    }
    if settings.max_samples == 0 || settings.max_samples > MAX_STEPS {
        return Err(format!("transient max samples must be 1 to {MAX_STEPS}"));
    }
    if settings.adaptive {
        return validate_adaptive(settings);
    }
    validate_fixed(settings)
}

fn validate_fixed(settings: &TransientSettings) -> Result<(), String> {
    if !settings.dt.is_finite() || settings.dt <= 0.0 {
        return Err("transient dt must be positive".to_string());
    }
    if settings.steps == 0 || settings.steps > settings.max_samples {
        return Err("transient steps exceed max samples".to_string());
    }
    if settings.dt * settings.steps as f64 > MAX_SECONDS {
        return Err(format!("transient duration exceeds {MAX_SECONDS} s"));
    }
    Ok(())
}

fn validate_adaptive(settings: &TransientSettings) -> Result<(), String> {
    if !settings.initial_dt.is_finite() || settings.initial_dt <= 0.0 {
        return Err("adaptive initial dt must be positive".to_string());
    }
    if !settings.min_dt.is_finite() || settings.min_dt <= 0.0 {
        return Err("adaptive min dt must be positive".to_string());
    }
    if !settings.max_dt.is_finite() || settings.max_dt < settings.min_dt {
        return Err("adaptive max dt must be at least min dt".to_string());
    }
    if !settings.tolerance.is_finite() || settings.tolerance <= 0.0 {
        return Err("adaptive tolerance must be positive".to_string());
    }
    Ok(())
}

fn trace(
    netlist: &Netlist,
    samples: &[TransientSample],
    settings: &TransientSettings,
) -> TransientTrace {
    let mut trace = TransientTrace::empty(metadata(netlist, settings));
    for sample in samples {
        trace.push(sample);
    }
    trace
}

/// Returns trace metadata for a netlist and transient settings.
pub(crate) fn metadata(netlist: &Netlist, settings: &TransientSettings) -> TraceMetadata {
    TraceMetadata {
        nodes: netlist.nodes.iter().map(|node| node.name.clone()).collect(),
        devices: devices(netlist),
        dt: trace_dt(settings),
        adaptive: settings.adaptive,
    }
}

/// Builds a fixed-step config for an incremental session.
pub(crate) fn fixed_config(settings: &TransientSettings) -> Result<TransientConfig, String> {
    if settings.adaptive {
        return Err("pause and step require fixed step".to_string());
    }
    validate_fixed(settings)?;
    Ok(TransientConfig {
        dt: settings.dt,
        steps: settings.steps,
        gmin: settings.gmin,
        use_dc_op: settings.use_dc_op,
        ..TransientConfig::default()
    })
}

fn trace_dt(settings: &TransientSettings) -> f64 {
    if settings.adaptive {
        return settings.initial_dt;
    }
    settings.dt
}

fn devices(netlist: &Netlist) -> Vec<TransientDevice> {
    netlist
        .components
        .iter()
        .map(|part| TransientDevice {
            label: part.label.clone(),
            part: part.part,
        })
        .collect()
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
