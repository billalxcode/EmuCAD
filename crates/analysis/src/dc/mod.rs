//! DC operating point and sweep analysis.

use crate::newton::{damp_solution, max_node_delta, solution_close};
use crate::system::SimulationSystem;
use kernel::{Circuit, Device, DeviceKind, SimError, Solution};

/// DC solver configuration.
#[derive(Clone, Debug)]
pub struct DcConfig {
    /// Maximum Newton iterations.
    pub max_iterations: usize,
    /// Relative convergence tolerance.
    pub rel_tol: f64,
    /// Absolute convergence tolerance.
    pub abs_tol: f64,
    /// Minimum conductance added to each node.
    pub gmin: f64,
    /// Starting conductance for gmin stepping.
    pub gmin_start: f64,
    /// Maximum node-voltage change per Newton step.
    pub max_step: f64,
}

impl Default for DcConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            rel_tol: 1e-6,
            abs_tol: 1e-9,
            gmin: 1e-12,
            gmin_start: 1e-3,
            max_step: 0.5,
        }
    }
}

/// A DC operating point result.
#[derive(Clone, Debug)]
pub struct DcResult {
    /// Converged solution.
    pub solution: Solution,
    /// Number of iterations used.
    pub iterations: usize,
    /// Maximum node-voltage change at convergence.
    pub residual: f64,
}

/// One point in a DC sweep.
#[derive(Clone, Debug)]
pub struct DcPoint {
    /// Sweep value.
    pub sweep: f64,
    /// Solution at this value.
    pub solution: Solution,
}

/// Solves the DC operating point of a circuit.
pub fn dc_operating_point(circuit: &Circuit, config: &DcConfig) -> Result<DcResult, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    let initial = system.zero_solution();
    match solve_dc_system(&mut system, initial, config) {
        Ok(result) => Ok(result),
        Err(error) => source_stepping(circuit, config).map_err(|_| error),
    }
}

/// Returns one terminal current per device at a DC solution.
pub fn dc_terminal_currents(
    circuit: &Circuit,
    result: &DcResult,
) -> Result<Vec<Option<f64>>, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    system.update(&result.solution)?;
    Ok(system.currents(&result.solution))
}

mod operating;
mod sweep;
#[cfg(test)]
mod tests;

pub(crate) use operating::solve_dc_system;
use operating::*;
pub use sweep::dc_sweep;
