//! Behavioral optically coupled isolator.

use crate::model::{
    DeviceModel, non_negative_parameter, pins_exact, positive_parameter, stamp_conductance,
    stamp_current_source,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A four-pin optocoupler: LED anode, LED cathode, collector, emitter.
pub struct OptocouplerModel {
    ctr: f64,
    forward_voltage: f64,
    led_resistance: f64,
    led_leakage: f64,
    vce_sat: f64,
    output_leakage: f64,
    pins: [NodeId; 4],
}

impl OptocouplerModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        let ctr = positive_parameter(params, "current_transfer_ratio", Some(1.0))?;
        let forward_voltage = positive_parameter(params, "forward_voltage", Some(1.2))?;
        let led_resistance = positive_parameter(params, "rs", Some(10.0))?;
        let led_leakage = non_negative_parameter(params, "led_leakage", 1e-9)?;
        let vce_sat = positive_parameter(params, "vce_sat", Some(0.2))?;
        let output_leakage = non_negative_parameter(params, "leakage", 1e-9)?;
        Ok(Self {
            ctr,
            forward_voltage,
            led_resistance,
            led_leakage,
            vce_sat,
            output_leakage,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn led_current(&self, voltage: f64) -> (f64, f64) {
        if voltage <= self.forward_voltage {
            return (self.led_leakage * voltage, self.led_leakage);
        }
        let current =
            self.led_leakage * voltage + (voltage - self.forward_voltage) / self.led_resistance;
        (current, self.led_leakage + 1.0 / self.led_resistance)
    }

    fn output_current(&self, voltage: f64, led_current: f64) -> (f64, f64) {
        if voltage <= 0.0 {
            return (self.output_leakage * voltage, self.output_leakage);
        }
        let knee = (-voltage / self.vce_sat).exp();
        let current = self.output_leakage * voltage + self.ctr * led_current * (1.0 - knee);
        let conductance = self.output_leakage + self.ctr * led_current * knee / self.vce_sat;
        (current, conductance)
    }
}

impl DeviceModel for OptocouplerModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let led_voltage = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let (led_current, led_conductance) = self.led_current(led_voltage);
        stamp_conductance(ctx, self.pins[0], self.pins[1], led_conductance)?;
        stamp_current_source(
            ctx,
            self.pins[0],
            self.pins[1],
            led_current - led_conductance * led_voltage,
        )?;
        let output_voltage = solution.voltage(self.pins[2]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let (current, conductance) = self.output_current(output_voltage, led_current.abs());
        stamp_conductance(ctx, self.pins[2], self.pins[3], conductance)?;
        stamp_current_source(
            ctx,
            self.pins[2],
            self.pins[3],
            current - conductance * output_voltage,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OptocouplerModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "u1",
            DeviceKind::Optocoupler,
            (1..=4)
                .map(|index| NodeId::new(u32::try_from(index).expect("pin")))
                .collect(),
            parameters,
        )
    }

    #[test]
    fn led_current_turns_on_above_forward_voltage() {
        let model = OptocouplerModel::from_instance(&instance(&[])).expect("model");
        let (off, _) = model.led_current(1.0);
        let (on, _) = model.led_current(2.0);
        assert!(off.abs() < 1e-6);
        assert!(on > 1e-3);
    }

    #[test]
    fn output_current_scales_with_ctr() {
        let model = OptocouplerModel::from_instance(&instance(&[("current_transfer_ratio", 2.0)]))
            .expect("model");
        let (current, _) = model.output_current(1.0, 1e-3);
        assert!(current > 1.5e-3, "current={current}");
    }

    #[test]
    fn output_leakage_is_small_when_led_is_off() {
        let model = OptocouplerModel::from_instance(&instance(&[])).expect("model");
        let (current, _) = model.output_current(5.0, 0.0);
        assert!(current.abs() < 1e-6, "current={current}");
    }
}
