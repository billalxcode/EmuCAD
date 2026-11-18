use super::*;

/// Sweeps a device parameter and solves the DC operating point at each step.
pub fn dc_sweep(
    circuit: &Circuit,
    device: &str,
    parameter: &str,
    start: f64,
    stop: f64,
    step: f64,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let sweep = SweepSpec {
        device,
        parameter,
        start,
        stop,
        step,
    };
    run_sweep(circuit, &sweep, config)
}

struct SweepSpec<'a> {
    device: &'a str,
    parameter: &'a str,
    start: f64,
    stop: f64,
    step: f64,
}

fn run_sweep(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    validate_sweep(sweep.start, sweep.stop, sweep.step)?;
    collect_sweep_points(circuit, sweep, config)
}

fn collect_sweep_points(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    config: &DcConfig,
) -> Result<Vec<DcPoint>, SimError> {
    let mut points = Vec::new();
    let mut previous = None;
    let mut value = sweep.start;
    while value <= sweep.stop + sweep.step * 1e-9 {
        ensure_sweep_capacity(points.len())?;
        let point = solve_sweep_point(circuit, sweep, value, config, previous.as_ref())?;
        previous = Some(point.solution.clone());
        points.push(point);
        value += sweep.step;
    }
    Ok(points)
}

fn ensure_sweep_capacity(len: usize) -> Result<(), SimError> {
    if len >= 1_000_000 {
        return Err(SimError::InvalidNetlist("sweep limit exceeded".to_string()));
    }
    Ok(())
}

fn solve_sweep_point(
    circuit: &Circuit,
    sweep: &SweepSpec<'_>,
    value: f64,
    config: &DcConfig,
    initial: Option<&Solution>,
) -> Result<DcPoint, SimError> {
    let mut swept = circuit.clone();
    swept.set_device_parameter(sweep.device, sweep.parameter, value)?;
    let result = match initial {
        Some(initial) => dc_operating_point_with_initial(&swept, initial.clone(), config)?,
        None => dc_operating_point(&swept, config)?,
    };
    Ok(DcPoint {
        sweep: value,
        solution: result.solution,
    })
}
fn validate_sweep(start: f64, stop: f64, step: f64) -> Result<(), SimError> {
    if !start.is_finite() || !stop.is_finite() || !step.is_finite() || step <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "sweep values must be finite and step must be positive".to_string(),
        ));
    }
    if stop < start {
        return Err(SimError::InvalidNetlist(
            "sweep stop must be greater than or equal to start".to_string(),
        ));
    }
    Ok(())
}
