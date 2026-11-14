use super::*;

pub(crate) fn parameter(params: &BTreeMap<String, f64>, name: &str, default: f64) -> f64 {
    params.get(name).copied().unwrap_or(default)
}

pub(crate) fn non_negative_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: f64,
) -> Result<f64, SimError> {
    let value = parameter(params, name, default);
    if !value.is_finite() || value < 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be non-negative"
        )));
    }
    Ok(value)
}

pub(crate) fn positive_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: Option<f64>,
) -> Result<f64, SimError> {
    let value = match default {
        Some(default) => parameter(params, name, default),
        None => params
            .get(name)
            .copied()
            .ok_or_else(|| SimError::InvalidNetlist(format!("missing parameter: {name}")))?,
    };
    if !value.is_finite() || value <= 0.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be positive"
        )));
    }
    Ok(value)
}

pub(crate) fn thermal_voltage(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
    let default = BOLTZMANN * temperature / ELEMENTARY_CHARGE;
    positive_parameter(params, "vt", Some(default))
}

pub(crate) fn depletion_capacitance(zero: f64, potential: f64, grading: f64, voltage: f64) -> f64 {
    if zero <= 0.0 {
        return 0.0;
    }
    let ratio = (1.0 - voltage / potential).max(0.1);
    zero / ratio.powf(grading)
}

/// Returns the critical junction voltage used by limiting.
pub(crate) fn junction_critical_voltage(saturation: f64, thermal_voltage: f64) -> f64 {
    thermal_voltage * (thermal_voltage / (std::f64::consts::SQRT_2 * saturation)).ln()
}

pub(crate) fn terminal_conductance(junction: f64, series_resistance: f64) -> f64 {
    if series_resistance <= 0.0 {
        junction
    } else {
        junction / (1.0 + series_resistance * junction)
    }
}

/// Limits a large junction voltage step.
pub(crate) fn pnjlim(voltage: f64, previous: f64, thermal: f64, critical: f64) -> f64 {
    if voltage <= critical || (voltage - previous).abs() <= 2.0 * thermal {
        return voltage;
    }
    if previous > 0.0 {
        let argument = 1.0 + (voltage - previous) / thermal;
        if argument > 0.0 {
            return previous + thermal * argument.ln();
        }
        return critical;
    }
    thermal * (voltage / thermal).ln()
}
