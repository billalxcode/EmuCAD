//! Behavioral actuator and display modules.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_current_source, terminal_conductance, thermal_voltage,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

const MAX_LED_EXPONENT: f64 = 60.0;

/// An LED module using an exponential junction and series resistance.
pub struct LedActuatorModel {
    saturation_current: f64,
    emission: f64,
    thermal_voltage: f64,
    series_resistance: f64,
    pins: [NodeId; 2],
}

impl LedActuatorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        let pins = pins_exact(instance, 2)?;
        Ok(Self {
            saturation_current: positive_parameter(params, "is", Some(1e-15))?,
            emission: positive_parameter(params, "n", Some(2.0))?,
            thermal_voltage: thermal_voltage(params)?,
            series_resistance: non_negative_parameter(params, "rs", 10.0)?,
            pins: [pins[0], pins[1]],
        })
    }

    fn junction_current(&self, voltage: f64) -> f64 {
        let scale = self.emission * self.thermal_voltage;
        self.saturation_current * ((voltage / scale).min(MAX_LED_EXPONENT).exp() - 1.0)
    }

    fn junction_conductance(&self, voltage: f64) -> f64 {
        let scale = self.emission * self.thermal_voltage;
        self.saturation_current / scale * (voltage / scale).min(MAX_LED_EXPONENT).exp()
    }

    fn junction_voltage(&self, terminal: f64) -> f64 {
        if self.series_resistance <= 0.0 {
            return terminal;
        }
        let mut low = terminal.min(0.0) - 1.0;
        let mut high = terminal.max(1.0) + 1.0;
        for _ in 0..60 {
            let middle = 0.5 * (low + high);
            let value = middle + self.series_resistance * self.junction_current(middle) - terminal;
            if value > 0.0 {
                high = middle;
            } else {
                low = middle;
            }
        }
        0.5 * (low + high)
    }
}

impl DeviceModel for LedActuatorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltage = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let junction = self.junction_voltage(voltage);
        let current = self.junction_current(junction);
        let conductance =
            terminal_conductance(self.junction_conductance(junction), self.series_resistance);
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        let constant = conductance * voltage - current;
        let a = ctx.node_index(self.pins[0]);
        let b = ctx.node_index(self.pins[1]);
        ctx.add_rhs_opt(a, constant)?;
        ctx.add_rhs_opt(b, -constant)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        let voltage = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        Some(self.junction_current(self.junction_voltage(voltage)))
    }
}

/// A two-terminal resistor in series with a controlled voltage source.
pub struct SeriesEmfModel {
    resistance: f64,
    coefficient: f64,
    input: f64,
    pins: [NodeId; 2],
}

impl SeriesEmfModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        input_name: &str,
        coefficient_name: &str,
        resistance: f64,
        coefficient: f64,
    ) -> Result<Self, SimError> {
        let params = instance.parameters();
        let pins = pins_exact(instance, 2)?;
        Ok(Self {
            resistance: positive_parameter(params, "resistance", Some(resistance))?,
            coefficient: positive_parameter(params, coefficient_name, Some(coefficient))?,
            input: finite_parameter(params, input_name, 0.0)?,
            pins: [pins[0], pins[1]],
        })
    }

    fn emf(&self) -> f64 {
        self.coefficient * self.input
    }

    fn current(&self, voltage: f64) -> f64 {
        (voltage - self.emf()) / self.resistance
    }
}

impl DeviceModel for SeriesEmfModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let conductance = 1.0 / self.resistance.max(1e-9);
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        stamp_current_source(ctx, self.pins[1], self.pins[0], self.emf() * conductance)?;
        Ok(())
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        let voltage = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        Some(self.current(voltage))
    }
}

/// A four-pin display module with a supply load and high-impedance inputs.
pub struct DisplayModuleModel {
    rated_voltage: f64,
    supply_current: f64,
    brightness: f64,
    input_leakage: f64,
    pins: [NodeId; 4],
}

impl DisplayModuleModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        let pins = pins_exact(instance, 4)?;
        Ok(Self {
            rated_voltage: positive_parameter(params, "rated_voltage", Some(5.0))?,
            supply_current: non_negative_parameter(params, "supply_current", 0.002)?,
            brightness: non_negative_parameter(params, "brightness", 1.0)?,
            input_leakage: non_negative_parameter(params, "input_leakage", 1e-6)?,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn supply_conductance(&self) -> f64 {
        self.supply_current * self.brightness / self.rated_voltage
    }
}

impl DeviceModel for DisplayModuleModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_conductance(ctx, self.pins[0], self.pins[1], self.supply_conductance())?;
        stamp_conductance(ctx, self.pins[2], self.pins[1], self.input_leakage)?;
        stamp_conductance(ctx, self.pins[3], self.pins[1], self.input_leakage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayModuleModel, LedActuatorModel, SeriesEmfModel};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, pins: usize, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        let nodes = (0..pins).map(|_| NodeId::GROUND).collect();
        DeviceInstance::new(DeviceId::new(0), "A1".to_string(), kind, nodes, parameters)
    }

    #[test]
    fn led_module_adds_led_defaults() {
        let model = LedActuatorModel::from_instance(&instance(
            DeviceKind::LedActuator,
            2,
            &[("is", 1e-15)],
        ));
        assert!(model.is_ok());
    }

    #[test]
    fn series_emf_current_uses_back_emf() {
        let model = SeriesEmfModel::from_instance(
            &instance(
                DeviceKind::DcMotor,
                2,
                &[("resistance", 10.0), ("speed", 100.0)],
            ),
            "speed",
            "back_emf_constant",
            10.0,
            0.01,
        )
        .expect("model");
        let current = model.current(5.0);
        assert!((current - 0.4).abs() < 1e-9);
    }

    #[test]
    fn display_supply_conductance_uses_brightness() {
        let model = DisplayModuleModel::from_instance(&instance(
            DeviceKind::DisplayModule,
            4,
            &[("supply_current", 0.002), ("rated_voltage", 5.0)],
        ))
        .expect("model");
        assert!((model.supply_conductance() - 0.0004).abs() < 1e-12);
    }
}
