//! Transient analysis using backward Euler.

use crate::newton::{NewtonConfig, damp_solution, max_node_delta, solution_close};
use crate::system::SimulationSystem;
use sim_core::{Circuit, NodeId, SimError, Solution, Time};
use sim_device::IntegrationMethod;

/// Transient solver configuration.
#[derive(Clone, Debug)]
pub struct TransientConfig {
    /// Time step in seconds.
    pub dt: f64,
    /// Number of steps.
    pub steps: usize,
    /// Minimum conductance added to each node.
    pub gmin: f64,
    /// Use a DC operating point as the initial state.
    pub use_dc_op: bool,
    /// Node voltage overrides applied after the initial state.
    pub initial_voltages: Vec<(NodeId, f64)>,
}

impl Default for TransientConfig {
    fn default() -> Self {
        Self {
            dt: 1e-6,
            steps: 100,
            gmin: 1e-12,
            use_dc_op: true,
            initial_voltages: Vec::new(),
        }
    }
}

/// One transient sample.
#[derive(Clone, Debug)]
pub struct TransientSample {
    /// Simulation time.
    pub time: Time,
    /// Solution at this time.
    pub solution: Solution,
    /// Maximum node-voltage change at convergence.
    pub residual: f64,
    /// Newton iterations used for this step.
    pub iterations: usize,
}

/// Runs a zero-initial-state transient simulation.
pub fn transient(
    circuit: &Circuit,
    config: &TransientConfig,
) -> Result<Vec<TransientSample>, SimError> {
    transient_with_method(circuit, config, IntegrationMethod::BackwardEuler)
}

/// Runs a transient simulation with an explicit integration method.
pub fn transient_with_method(
    circuit: &Circuit,
    config: &TransientConfig,
    method: IntegrationMethod,
) -> Result<Vec<TransientSample>, SimError> {
    transient_with_solver(circuit, config, method, &NewtonConfig::default())
}

/// Runs a transient simulation with an explicit integration method and Newton settings.
pub fn transient_with_solver(
    circuit: &Circuit,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<Vec<TransientSample>, SimError> {
    validate_config(config)?;
    validate_newton(newton)?;
    let mut system = SimulationSystem::new(circuit)?;
    let solution = initial_solution(&mut system, config)?;
    system.update(&solution)?;
    run_steps(&mut system, solution, config, method, newton)
}

pub(crate) fn initial_solution(
    system: &mut SimulationSystem,
    config: &TransientConfig,
) -> Result<Solution, SimError> {
    let initial = system.zero_solution();
    if !config.use_dc_op {
        return Ok(apply_initial_voltages(initial, &config.initial_voltages));
    }
    let dc_config = crate::DcConfig {
        gmin: config.gmin,
        ..crate::DcConfig::default()
    };
    let solution = crate::dc::solve_dc_system(system, initial, &dc_config)?.solution;
    Ok(apply_initial_voltages(solution, &config.initial_voltages))
}

fn apply_initial_voltages(mut solution: Solution, overrides: &[(NodeId, f64)]) -> Solution {
    for (node, voltage) in overrides {
        if let Some(slot) = solution.node_voltages.get_mut(node.raw() as usize) {
            *slot = *voltage;
        }
    }
    solution
}

mod steps;
#[cfg(test)]
mod tests;
mod validation;

use steps::run_steps;
pub(crate) use steps::solve_step;
use validation::validate_config;
pub(crate) use validation::validate_newton;
