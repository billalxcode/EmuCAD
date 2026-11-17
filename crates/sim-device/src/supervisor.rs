//! Behavioral references and supply supervisors.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_current_source, stamp_output, stamp_voltage_branch,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A three-pin voltage reference with quiescent supply current.
pub struct VoltageReferenceModel {
    id: sim_core::DeviceId,
    voltage: f64,
    quiescent_current: f64,
    pins: [NodeId; 3],
}

impl VoltageReferenceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        Ok(Self {
            id: instance.id(),
            voltage: finite_parameter(params, "threshold", 2.5)?,
            quiescent_current: non_negative_parameter(params, "quiescent_current", 1e-3)?,
            pins: [pins[0], pins[1], pins[2]],
        })
    }
}

impl DeviceModel for VoltageReferenceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_current_source(ctx, self.pins[0], self.pins[1], self.quiescent_current)?;
        stamp_voltage_branch(ctx, self.id, self.pins[2], self.pins[1], self.voltage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A three-pin current reference with quiescent supply current.
pub struct CurrentReferenceModel {
    current: f64,
    quiescent_current: f64,
    pins: [NodeId; 3],
}

impl CurrentReferenceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        Ok(Self {
            current: finite_parameter(params, "output_current", 1e-3)?,
            quiescent_current: non_negative_parameter(params, "quiescent_current", 1e-3)?,
            pins: [pins[0], pins[1], pins[2]],
        })
    }
}

impl DeviceModel for CurrentReferenceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_current_source(ctx, self.pins[0], self.pins[1], self.quiescent_current)?;
        stamp_current_source(ctx, self.pins[1], self.pins[2], self.current)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }
}

/// A supply supervisor with hysteresis and selectable polarity.
pub struct SupervisorModel {
    threshold: f64,
    hysteresis: f64,
    active_low: bool,
    output_resistance: f64,
    on: bool,
    pins: [NodeId; 3],
}

impl SupervisorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        Ok(Self {
            threshold: finite_parameter(params, "threshold", 2.5)?,
            hysteresis: non_negative_parameter(params, "hysteresis", 0.1)?,
            active_low: parameter_bool(params, "active_low", false)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            on: false,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn output(&mut self, supply: f64) -> f64 {
        if supply > self.threshold {
            self.on = true;
        }
        if supply < self.threshold - self.hysteresis {
            self.on = false;
        }
        let high = if self.active_low {
            0.0
        } else {
            supply.max(0.0)
        };
        let low = if self.active_low {
            supply.max(0.0)
        } else {
            0.0
        };
        if self.on { high } else { low }
    }
}

fn parameter_bool(
    params: &std::collections::BTreeMap<String, f64>,
    name: &str,
    default: bool,
) -> Result<bool, SimError> {
    let value = finite_parameter(params, name, if default { 1.0 } else { 0.0 })?;
    if value != 0.0 && value != 1.0 {
        return Err(SimError::InvalidNetlist(format!(
            "parameter {name} must be zero or one"
        )));
    }
    Ok(value == 1.0)
}

impl DeviceModel for SupervisorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let output = self.output(supply);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            output,
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrentReferenceModel, SupervisorModel, VoltageReferenceModel};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "U1".to_string(),
            kind,
            vec![NodeId::new(1), NodeId::GROUND, NodeId::new(2)],
            parameters,
        )
    }

    #[test]
    fn voltage_reference_builds() {
        assert!(
            VoltageReferenceModel::from_instance(&instance(
                DeviceKind::VoltageReference,
                &[("threshold", 2.5)],
            ))
            .is_ok()
        );
    }

    #[test]
    fn current_reference_builds() {
        assert!(
            CurrentReferenceModel::from_instance(&instance(
                DeviceKind::CurrentReference,
                &[("output_current", 1e-3)],
            ))
            .is_ok()
        );
    }

    #[test]
    fn supervisor_rejects_non_boolean_active_low() {
        let result = SupervisorModel::from_instance(&instance(
            DeviceKind::Supervisor,
            &[("active_low", 2.0)],
        ));
        assert!(result.is_err());
    }
}
