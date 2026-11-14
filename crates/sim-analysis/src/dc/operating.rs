use super::*;

pub(super) fn source_stepping(circuit: &Circuit, config: &DcConfig) -> Result<DcResult, SimError> {
    let steps = 20;
    let mut solution = None;
    let mut result = None;
    for step in 1..=steps {
        let scale = step as f64 / steps as f64;
        let scaled = scaled_sources(circuit, scale)?;
        let mut system = SimulationSystem::new(&scaled)?;
        let initial = match solution.take() {
            Some(value) => value,
            None => system.zero_solution(),
        };
        let solved = solve_dc_system(&mut system, initial, config)?;
        solution = Some(solved.solution.clone());
        result = Some(solved);
    }
    result.ok_or_else(|| SimError::InvalidNetlist("source stepping failed".to_string()))
}

fn scaled_sources(circuit: &Circuit, scale: f64) -> Result<Circuit, SimError> {
    let mut scaled = circuit.snapshot();
    for device in circuit.devices() {
        let parameter = match device.kind() {
            DeviceKind::VoltageSource => "voltage",
            DeviceKind::CurrentSource => "current",
            _ => continue,
        };
        if let Some(value) = device.parameters().get(parameter).copied() {
            scaled.set_device_parameter(device.name(), parameter, value * scale)?;
        }
    }
    Ok(scaled)
}

pub(super) fn dc_operating_point_with_initial(
    circuit: &Circuit,
    initial: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    let mut system = SimulationSystem::new(circuit)?;
    solve_dc_system(&mut system, initial, config)
}
pub(crate) fn solve_dc_system(
    system: &mut SimulationSystem,
    initial: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    solve_with_gmin_stepping(system, initial, config)
}

fn solve_with_gmin_stepping(
    system: &mut SimulationSystem,
    solution: Solution,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    if system.is_linear() {
        return solve_dc(system, solution, config.gmin, config);
    }
    let mut current = solution;
    let mut gmin = config.gmin_start.max(config.gmin);
    while gmin > config.gmin {
        current = solve_dc(system, current, gmin, config)?.solution;
        gmin = (gmin * 0.1).max(config.gmin);
    }
    solve_dc(system, current, config.gmin, config)
}
fn solve_dc(
    system: &mut SimulationSystem,
    initial: Solution,
    gmin: f64,
    config: &DcConfig,
) -> Result<DcResult, SimError> {
    let mut solution = initial;
    for iteration in 0..config.max_iterations {
        let (next, residual, converged) = solve_iteration(system, &solution, gmin, config)?;
        solution = next;
        if converged {
            return finish_dc(system, solution, iteration + 1, residual);
        }
    }
    Err(SimError::ConvergenceFailure {
        iteration: config.max_iterations,
    })
}

fn solve_iteration(
    system: &mut SimulationSystem,
    solution: &Solution,
    gmin: f64,
    config: &DcConfig,
) -> Result<(Solution, f64, bool), SimError> {
    let raw = system.solve(solution, None, gmin)?;
    let next = damp_solution(solution, &raw, config.max_step);
    let residual = max_node_delta(solution, &next);
    let converged = solution_close(solution, &next, config.rel_tol, config.abs_tol);
    Ok((next, residual, converged))
}

fn finish_dc(
    system: &mut SimulationSystem,
    solution: Solution,
    iterations: usize,
    residual: f64,
) -> Result<DcResult, SimError> {
    system.update(&solution)?;
    Ok(DcResult {
        solution,
        iterations,
        residual,
    })
}
