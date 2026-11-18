use super::*;

pub(crate) fn validate_newton(config: &NewtonConfig) -> Result<(), SimError> {
    if config.max_iterations == 0 {
        return Err(SimError::InvalidNetlist(
            "transient max iterations must be positive".to_string(),
        ));
    }
    validate_tolerances(config)?;
    validate_step(config)
}

fn validate_tolerances(config: &NewtonConfig) -> Result<(), SimError> {
    if !config.rel_tol.is_finite() || config.rel_tol <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient rel_tol must be positive".to_string(),
        ));
    }
    if !config.abs_tol.is_finite() || config.abs_tol <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient abs_tol must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_step(config: &NewtonConfig) -> Result<(), SimError> {
    if !config.max_step.is_finite() || config.max_step < 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient max_step must be non-negative".to_string(),
        ));
    }
    Ok(())
}
pub(super) fn validate_config(config: &TransientConfig) -> Result<(), SimError> {
    if !config.dt.is_finite() || config.dt <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient dt must be positive".to_string(),
        ));
    }
    if !config.gmin.is_finite() || config.gmin < 0.0 {
        return Err(SimError::InvalidNetlist(
            "transient gmin must be non-negative".to_string(),
        ));
    }
    Ok(())
}
