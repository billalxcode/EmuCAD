//! AC small-signal analysis with DC bias linearization.

use crate::dc::{DcConfig, dc_operating_point};
use crate::system::{SimulationSystem, build_branch_map, build_node_map, count_nodes};
use sim_core::{Circuit, Complex64, ComplexMatrix, Device, DeviceKind, NodeId, SimError, Solution};
use sim_device::{bjt_capacitances, diode_capacitance, jfet_capacitances, mosfet_capacitances};

/// AC solution at one frequency.
#[derive(Clone, Debug)]
pub struct AcSolution {
    frequency: f64,
    node_map: Vec<Option<usize>>,
    branch_map: Vec<Option<usize>>,
    node_voltages: Vec<Complex64>,
    branch_currents: Vec<Complex64>,
}

impl AcSolution {
    /// Returns the analysis frequency.
    #[must_use]
    pub const fn frequency(&self) -> f64 {
        self.frequency
    }

    /// Returns a node voltage.
    #[must_use]
    pub fn node_voltage(&self, node: NodeId) -> Option<Complex64> {
        let index = *self.node_map.get(node.raw() as usize)?.as_ref()?;
        self.node_voltages.get(index).copied()
    }

    /// Returns a node magnitude.
    #[must_use]
    pub fn magnitude(&self, node: NodeId) -> Option<f64> {
        self.node_voltage(node).map(Complex64::magnitude)
    }

    /// Returns a node phase in degrees.
    #[must_use]
    pub fn phase_degrees(&self, node: NodeId) -> Option<f64> {
        self.node_voltage(node)
            .map(|value| value.phase().to_degrees())
    }

    /// Returns a branch current.
    #[must_use]
    pub fn branch_current(&self, device: usize) -> Option<Complex64> {
        let index = *self.branch_map.get(device)?.as_ref()?;
        self.branch_currents.get(index).copied()
    }
}

/// Runs AC analysis at one frequency.
pub fn ac_analysis(circuit: &Circuit, frequency: f64) -> Result<AcSolution, SimError> {
    validate_frequency(frequency)?;
    ensure_supported(circuit)?;
    let bias = dc_operating_point(circuit, &DcConfig::default())?.solution;
    let node_map = build_node_map(circuit);
    let node_count = count_nodes(&node_map);
    let (branch_map, branch_count) = build_branch_map(circuit);
    let size = node_count + branch_count;
    let mut matrix = bias_matrix(circuit, &bias, size)?;
    let reactive = ReactiveContext::new(&bias, frequency, &node_map, &branch_map);
    add_reactive(circuit, &reactive, &mut matrix)?;
    let mut rhs = vec![Complex64::default(); size];
    stamp_sources(circuit, &node_map, &branch_map, &mut rhs)?;
    let values = matrix.solve(&rhs)?;
    Ok(build_solution(
        frequency, node_map, branch_map, values, node_count,
    ))
}
fn build_solution(
    frequency: f64,
    node_map: Vec<Option<usize>>,
    branch_map: Vec<Option<usize>>,
    values: Vec<Complex64>,
    node_count: usize,
) -> AcSolution {
    let branch_map = branch_map
        .into_iter()
        .map(|slot| slot.map(|index| index - node_count))
        .collect();
    AcSolution {
        frequency,
        node_map,
        branch_map,
        node_voltages: values[..node_count].to_vec(),
        branch_currents: values[node_count..].to_vec(),
    }
}

mod reactive;
mod stamps;
mod support;
mod sweep;
#[cfg(test)]
mod tests;

pub use sweep::ac_sweep;
pub(crate) use sweep::sweep_frequencies;

use reactive::{ReactiveContext, add_reactive};
use stamps::{add_entry, branch_index, node_pair, parameter, stamp_pair, stamp_sources};
use support::{bias_matrix, ensure_supported, validate_frequency};
