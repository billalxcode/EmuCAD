//! Behavioral motor driver modules.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_output,
};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A four-pin single-channel motor driver.
pub struct MotorDriverModel {
    threshold: f64,
    hysteresis: f64,
    on_resistance: f64,
    off_resistance: f64,
    on: bool,
    pins: [NodeId; 4],
}

impl MotorDriverModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        Ok(Self {
            threshold: finite_parameter(params, "threshold", 2.5)?,
            hysteresis: non_negative_parameter(params, "hysteresis", 0.1)?,
            on_resistance: positive_parameter(params, "on_resistance", Some(0.5))?,
            off_resistance: positive_parameter(params, "off_resistance", Some(1e6))?,
            on: false,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn output(&mut self, supply: f64, control: f64) -> f64 {
        if control > self.threshold {
            self.on = true;
        }
        if control < self.threshold - self.hysteresis {
            self.on = false;
        }
        if self.on { supply.max(0.0) } else { 0.0 }
    }

    fn output_resistance(&self) -> f64 {
        if self.on {
            self.on_resistance
        } else {
            self.off_resistance
        }
    }
}

impl DeviceModel for MotorDriverModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let control = solution.voltage(self.pins[2]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let output = self.output(supply, control);
        stamp_output(
            ctx,
            self.pins[3],
            self.pins[1],
            output,
            self.output_resistance(),
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MotorDriverModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "U1".to_string(),
            DeviceKind::MotorDriver,
            vec![
                NodeId::new(1),
                NodeId::GROUND,
                NodeId::new(2),
                NodeId::new(3),
            ],
            parameters,
        )
    }

    #[test]
    fn motor_driver_builds_from_four_pins() {
        assert!(MotorDriverModel::from_instance(&instance(&[])).is_ok());
    }

    #[test]
    fn motor_driver_rejects_zero_on_resistance() {
        let result = MotorDriverModel::from_instance(&instance(&[("on_resistance", 0.0)]));
        assert!(result.is_err());
    }
}
