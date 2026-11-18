//! Shichman-Hodges Level 1 MOSFET model.

use crate::model::{
    DERIVATIVE_STEP, DeviceModel, Linearized, finite_difference, non_negative_parameter, parameter,
    pins_three_or_four, positive_parameter, stamp_capacitor, stamp_four_terminal,
};
use kernel::{
    Device, DeviceInstance, DeviceKind, MnaContext, NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// Level 1 MOSFET parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MosfetParams {
    vto: f64,
    kp: f64,
    lambda: f64,
    width: f64,
    length: f64,
    gamma: f64,
    phi: f64,
    cox: f64,
    cgso: f64,
    cgdo: f64,
}

impl MosfetParams {
    /// Creates a parameter set.
    #[must_use]
    pub fn new(vto: f64, kp: f64, lambda: f64, width: f64, length: f64) -> Self {
        Self {
            vto,
            kp,
            lambda,
            width,
            length,
            gamma: 0.0,
            phi: 0.7,
            cox: 0.0,
            cgso: 0.0,
            cgdo: 0.0,
        }
    }

    /// Builds parameters from a device parameter map.
    pub fn from_map(params: &BTreeMap<String, f64>) -> Result<Self, SimError> {
        let vto = threshold_from_map(params)?;
        let gamma = finite_parameter(params, "gamma", 0.0)?;
        let (cox, cgso, cgdo) = capacitance_from_map(params)?;
        Ok(Self {
            vto,
            kp: positive_parameter(params, "kp", Some(1e-3))?,
            lambda: parameter(params, "lambda", 0.0),
            width: positive_parameter(params, "w", Some(1e-6))?,
            length: positive_parameter(params, "l", Some(1e-6))?,
            gamma,
            phi: positive_parameter(params, "phi", Some(0.7))?,
            cox,
            cgso,
            cgdo,
        })
    }
    /// Returns the threshold voltage.
    #[must_use]
    pub const fn vto(self) -> f64 {
        self.vto
    }

    /// Returns the transconductance parameter.
    #[must_use]
    pub const fn kp(self) -> f64 {
        self.kp
    }

    /// Returns the channel-length modulation parameter.
    #[must_use]
    pub const fn lambda(self) -> f64 {
        self.lambda
    }

    /// Returns the channel width.
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Returns the channel length.
    #[must_use]
    pub const fn length(self) -> f64 {
        self.length
    }

    /// Returns the body effect parameter.
    #[must_use]
    pub const fn gamma(self) -> f64 {
        self.gamma
    }

    /// Returns the surface potential.
    #[must_use]
    pub const fn phi(self) -> f64 {
        self.phi
    }
}

fn finite_parameter(
    params: &BTreeMap<String, f64>,
    name: &str,
    default: f64,
) -> Result<f64, SimError> {
    let value = parameter(params, name, default);
    if !value.is_finite() {
        return Err(SimError::InvalidNetlist(format!(
            "mosfet {name} must be finite"
        )));
    }
    Ok(value)
}

fn threshold_from_map(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let temperature = parameter(params, "temp", NOMINAL_TEMPERATURE);
    let tcv = parameter(params, "tcv", 0.0);
    let vto = parameter(params, "vto", 0.7) + tcv * (temperature - NOMINAL_TEMPERATURE);
    if !vto.is_finite() {
        return Err(SimError::InvalidNetlist(
            "mosfet vto must be finite".to_string(),
        ));
    }
    Ok(vto)
}

fn capacitance_from_map(params: &BTreeMap<String, f64>) -> Result<(f64, f64, f64), SimError> {
    Ok((
        non_negative_parameter(params, "cox", 0.0)?,
        non_negative_parameter(params, "cgso", 0.0)?,
        non_negative_parameter(params, "cgdo", 0.0)?,
    ))
}

/// A three-terminal or four-terminal MOSFET model.
mod model;
#[cfg(test)]
mod tests;

pub use model::{MosfetModel, mosfet_capacitances};
#[cfg(test)]
use model::{nmos_current_body, threshold};
