use super::*;

pub(super) fn stamp_sources(
    circuit: &Circuit,
    nodes: &[Option<usize>],
    branches: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    for (index, device) in circuit.devices().iter().enumerate() {
        match device.kind() {
            DeviceKind::VoltageSource => {
                stamp_voltage_rhs(device, index, branches, rhs)?;
            }
            DeviceKind::CurrentSource => {
                stamp_current_rhs(device, nodes, rhs)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn stamp_voltage_rhs(
    device: &impl Device,
    index: usize,
    branches: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    let branch = branch_index(branches, index)?;
    rhs[branch] = source_value(device)?;
    Ok(())
}

fn stamp_current_rhs(
    device: &impl Device,
    nodes: &[Option<usize>],
    rhs: &mut [Complex64],
) -> Result<(), SimError> {
    let value = source_value(device)?;
    let pair = node_pair(nodes, device.pins());
    add_rhs(rhs, pair[0], -value)?;
    add_rhs(rhs, pair[1], value)
}
pub(super) fn branch_index(branches: &[Option<usize>], device: usize) -> Result<usize, SimError> {
    branches
        .get(device)
        .and_then(|value| *value)
        .ok_or_else(|| SimError::Internal("missing AC branch".to_string()))
}

pub(super) fn stamp_pair(
    matrix: &mut ComplexMatrix,
    pair: [Option<usize>; 2],
    value: Complex64,
) -> Result<(), SimError> {
    let negative = -value;
    add_entry(matrix, pair[0], pair[0], value)?;
    add_entry(matrix, pair[0], pair[1], negative)?;
    add_entry(matrix, pair[1], pair[0], negative)?;
    add_entry(matrix, pair[1], pair[1], value)
}

pub(super) fn add_entry(
    matrix: &mut ComplexMatrix,
    row: Option<usize>,
    col: Option<usize>,
    value: Complex64,
) -> Result<(), SimError> {
    if let (Some(row), Some(col)) = (row, col) {
        matrix.add(row, col, value)?;
    }
    Ok(())
}

fn add_rhs(rhs: &mut [Complex64], index: Option<usize>, value: Complex64) -> Result<(), SimError> {
    if let Some(index) = index {
        let slot = rhs
            .get_mut(index)
            .ok_or(SimError::IndexOutOfBounds { row: index, col: 0 })?;
        *slot = *slot + value;
    }
    Ok(())
}

pub(super) fn node_pair(nodes: &[Option<usize>], pins: &[NodeId]) -> [Option<usize>; 2] {
    [nodes[pins[0].raw() as usize], nodes[pins[1].raw() as usize]]
}

pub(super) fn parameter(device: &impl Device, name: &str) -> Result<f64, SimError> {
    let value = device
        .parameters()
        .get(name)
        .copied()
        .ok_or_else(|| SimError::InvalidNetlist(format!("missing parameter: {name}")))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be positive"
        )));
    }
    Ok(value)
}

fn source_value(device: &impl Device) -> Result<Complex64, SimError> {
    let magnitude = device
        .parameters()
        .get("ac")
        .copied()
        .or_else(|| device.parameters().get("voltage").copied())
        .or_else(|| device.parameters().get("current").copied())
        .unwrap_or(1.0);
    let phase = device.parameters().get("ac_phase").copied().unwrap_or(0.0);
    let radians = phase.to_radians();
    Ok(Complex64::new(
        magnitude * radians.cos(),
        magnitude * radians.sin(),
    ))
}
