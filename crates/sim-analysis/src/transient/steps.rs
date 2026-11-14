use super::*;

pub(super) fn run_steps(
    system: &mut SimulationSystem,
    solution: Solution,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<Vec<TransientSample>, SimError> {
    let step = step_time(config.dt)?;
    let mut time = Time::ZERO;
    let mut samples = Vec::with_capacity(config.steps);
    let mut current = solution;
    for _ in 0..config.steps {
        let (next, residual, iterations) = step_sample(system, &current, config, method, newton)?;
        time = next_time(time, step)?;
        samples.push(sample(time, &next, residual, iterations));
        current = next;
    }
    Ok(samples)
}

fn step_sample(
    system: &mut SimulationSystem,
    current: &Solution,
    config: &TransientConfig,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, usize), SimError> {
    let (next, residual, iterations) =
        solve_step(system, current, config.dt, config.gmin, method, newton)?;
    system.update(&next)?;
    Ok((next, residual, iterations))
}

pub(crate) fn solve_step(
    system: &mut SimulationSystem,
    solution: &Solution,
    dt: f64,
    gmin: f64,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, usize), SimError> {
    let mut current = solution.clone();
    for iteration in 0..newton.max_iterations {
        let (next, residual, converged) =
            newton_iteration(system, &current, dt, gmin, method, newton)?;
        current = next;
        if converged {
            return Ok((current, residual, iteration + 1));
        }
    }
    Err(convergence_failure(newton))
}

fn convergence_failure(newton: &NewtonConfig) -> SimError {
    SimError::ConvergenceFailure {
        iteration: newton.max_iterations,
    }
}

fn newton_iteration(
    system: &mut SimulationSystem,
    current: &Solution,
    dt: f64,
    gmin: f64,
    method: IntegrationMethod,
    newton: &NewtonConfig,
) -> Result<(Solution, f64, bool), SimError> {
    let raw = system.solve_with_method(current, Some(dt), gmin, method)?;
    let next = damp_solution(current, &raw, newton.max_step);
    let residual = max_node_delta(current, &next);
    let converged = solution_close(current, &next, newton.rel_tol, newton.abs_tol);
    Ok((next, residual, converged))
}

fn next_time(time: Time, step: Time) -> Result<Time, SimError> {
    time.checked_add(step)
        .ok_or_else(|| SimError::Internal("transient time overflow".to_string()))
}

fn sample(time: Time, solution: &Solution, residual: f64, iterations: usize) -> TransientSample {
    TransientSample {
        time,
        solution: solution.clone(),
        residual,
        iterations,
    }
}

fn step_time(dt: f64) -> Result<Time, SimError> {
    Time::from_secs_f64(dt)
        .ok_or_else(|| SimError::InvalidNetlist("transient dt is out of range".to_string()))
}
