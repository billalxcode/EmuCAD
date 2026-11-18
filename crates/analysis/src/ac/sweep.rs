use super::*;

/// Sweeps frequency logarithmically.
pub fn ac_sweep(
    circuit: &Circuit,
    start: f64,
    stop: f64,
    points: usize,
) -> Result<Vec<AcSolution>, SimError> {
    let frequencies = sweep_frequencies(start, stop, points)?;
    frequencies
        .into_iter()
        .map(|frequency| ac_analysis(circuit, frequency))
        .collect()
}

pub(crate) fn sweep_frequencies(
    start: f64,
    stop: f64,
    points: usize,
) -> Result<Vec<f64>, SimError> {
    validate_sweep(start, stop, points)?;
    let ratio = (stop / start).powf(1.0 / (points - 1) as f64);
    let mut frequencies = Vec::with_capacity(points);
    let mut frequency = start;
    for _ in 0..points {
        frequencies.push(frequency);
        frequency *= ratio;
    }
    Ok(frequencies)
}
fn validate_sweep(start: f64, stop: f64, points: usize) -> Result<(), SimError> {
    validate_frequency(start)?;
    if !stop.is_finite() || stop < start {
        return Err(SimError::InvalidNetlist(
            "AC sweep range is invalid".to_string(),
        ));
    }
    if points < 2 {
        return Err(SimError::InvalidNetlist(
            "AC sweep needs at least two points".to_string(),
        ));
    }
    Ok(())
}
