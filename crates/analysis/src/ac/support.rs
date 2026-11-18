use super::*;

pub(super) fn ensure_supported(circuit: &Circuit) -> Result<(), SimError> {
    for device in circuit.devices() {
        if supported(device.kind()) {
            continue;
        }
        return Err(SimError::UnsupportedDevice(format!(
            "AC does not support device: {}",
            device.kind().name()
        )));
    }
    Ok(())
}

fn supported(kind: &DeviceKind) -> bool {
    supported_element(kind) || supported_semiconductor(kind) || supported_controlled(kind)
}

fn supported_element(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Resistor
            | DeviceKind::Capacitor
            | DeviceKind::Inductor
            | DeviceKind::VoltageSource
            | DeviceKind::CurrentSource
    )
}

fn supported_semiconductor(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Diode
            | DeviceKind::Nmos
            | DeviceKind::Pmos
            | DeviceKind::Npn
            | DeviceKind::Pnp
            | DeviceKind::JfetN
            | DeviceKind::JfetP
    )
}

fn supported_controlled(kind: &DeviceKind) -> bool {
    matches!(
        kind,
        DeviceKind::Vcvs
            | DeviceKind::Vccs
            | DeviceKind::Cccs
            | DeviceKind::Ccvs
            | DeviceKind::Switch
    )
}

pub(super) fn bias_matrix(
    circuit: &Circuit,
    bias: &kernel::Solution,
    size: usize,
) -> Result<ComplexMatrix, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    let context = system.stamp_dc(bias)?;
    let source = context.matrix();
    if source.rows() != size || source.cols() != size {
        return Err(SimError::Internal("AC bias matrix mismatch".to_string()));
    }
    copy_matrix(source, size)
}

fn copy_matrix(source: &kernel::DenseMatrix, size: usize) -> Result<ComplexMatrix, SimError> {
    let mut matrix = ComplexMatrix::zeros(size)?;
    for row in 0..size {
        for col in 0..size {
            let value = source.get(row, col).unwrap_or(0.0);
            if value != 0.0 {
                matrix.add(row, col, Complex64::from_real(value))?;
            }
        }
    }
    Ok(matrix)
}
pub(super) fn validate_frequency(frequency: f64) -> Result<(), SimError> {
    if !frequency.is_finite() || frequency <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "AC frequency must be positive".to_string(),
        ));
    }
    Ok(())
}
