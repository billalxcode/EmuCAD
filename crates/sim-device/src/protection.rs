//! Protection devices: voltage clamps and current-triggered fuses.

use crate::model::{
    DeviceModel, non_negative_parameter, parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_current_source,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A symmetric nonlinear voltage clamp.
pub struct VaristorModel {
    clamp_voltage: f64,
    clamp_current: f64,
    alpha: f64,
    leakage: f64,
    current: f64,
    pins: [NodeId; 2],
}

impl VaristorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let clamp_voltage = positive_parameter(instance.parameters(), "clamp_voltage", Some(6.0))?;
        let clamp_current = positive_parameter(instance.parameters(), "clamp_current", Some(1.0))?;
        let alpha = positive_parameter(instance.parameters(), "alpha", Some(4.0))?;
        let leakage = non_negative_parameter(instance.parameters(), "leakage", 1e-9)?;
        Ok(Self {
            clamp_voltage,
            clamp_current,
            alpha,
            leakage,
            current: 0.0,
            pins: [pins[0], pins[1]],
        })
    }

    fn evaluate(&self, voltage: f64) -> (f64, f64) {
        let magnitude = voltage.abs();
        if magnitude <= self.clamp_voltage {
            return (self.leakage * voltage, self.leakage);
        }
        let excess = magnitude / self.clamp_voltage - 1.0;
        let nonlinear = self.clamp_current * excess.powf(self.alpha);
        let current = self.leakage * voltage + voltage.signum() * nonlinear;
        let slope =
            self.alpha * self.clamp_current * excess.powf(self.alpha - 1.0) / self.clamp_voltage;
        (current, self.leakage + slope)
    }
}

impl DeviceModel for VaristorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltage = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let (current, conductance) = self.evaluate(voltage);
        let source = current - conductance * voltage;
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        stamp_current_source(ctx, self.pins[0], self.pins[1], source)?;
        self.current = current;
        Ok(())
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }
}

/// A current-triggered fuse or breaker.
pub struct FuseModel {
    resistance: f64,
    trip_current: f64,
    trip_time: f64,
    reset: bool,
    heat: f64,
    open: bool,
    dt_last: f64,
    current: f64,
    pins: [NodeId; 2],
}

impl FuseModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let resistance = positive_parameter(instance.parameters(), "resistance", Some(0.1))?;
        let trip_current = positive_parameter(instance.parameters(), "trip_current", Some(1.0))?;
        let trip_time = positive_parameter(instance.parameters(), "trip_time", Some(1e-3))?;
        let reset = parameter(instance.parameters(), "reset", 0.0) != 0.0;
        Ok(Self {
            resistance,
            trip_current,
            trip_time,
            reset,
            heat: 0.0,
            open: false,
            dt_last: 0.0,
            current: 0.0,
            pins: [pins[0], pins[1]],
        })
    }

    fn conductance(&self) -> f64 {
        if self.open {
            return 1e-12;
        }
        1.0 / self.resistance
    }

    fn update_heat(&mut self, current: f64, dt: f64) {
        let over = (current.abs() - self.trip_current).max(0.0) / self.trip_current;
        if !self.open {
            self.heat += over * dt / self.trip_time;
            self.heat = (self.heat - dt * 0.001).clamp(0.0, 1.0);
            self.open = self.heat >= 1.0;
            return;
        }
        if self.reset {
            self.heat = (self.heat - dt / self.trip_time * 0.1).max(0.0);
            self.open = self.heat > 0.0;
        }
    }
}

impl DeviceModel for FuseModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt_last = dt.unwrap_or(0.0);
        stamp_conductance(ctx, self.pins[0], self.pins[1], self.conductance())
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        self.current = (first - second) * self.conductance();
        if self.dt_last > 0.0 {
            self.update_heat(self.current, self.dt_last);
        }
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::{FuseModel, VaristorModel};
    use crate::model::DeviceModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, MnaContext, NodeId, Solution};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)], kind: DeviceKind) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "d1",
            kind,
            vec![NodeId::new(1), NodeId::GROUND],
            parameters,
        )
    }

    fn varistor(params: &[(&str, f64)]) -> VaristorModel {
        VaristorModel::from_instance(&instance(params, DeviceKind::Varistor)).expect("model")
    }

    fn fuse(params: &[(&str, f64)]) -> FuseModel {
        FuseModel::from_instance(&instance(params, DeviceKind::Fuse)).expect("model")
    }

    fn stamp_fuse(model: &mut FuseModel) -> MnaContext {
        let mut ctx = MnaContext::new(1, 0, vec![None, Some(0)], vec![]).expect("ctx");
        model
            .stamp(&mut ctx, &Solution::zero(2, 0), Some(1e-4))
            .expect("stamp");
        ctx
    }

    #[test]
    fn varistor_clamps_above_threshold() {
        let model = varistor(&[]);
        let (current, _) = model.evaluate(6.0);
        assert!(current.abs() < 1e-3);
        let (current, _) = model.evaluate(12.0);
        assert!(current > 1.0, "current={current}");
    }

    #[test]
    fn varistor_rejects_bad_alpha() {
        assert!(
            VaristorModel::from_instance(&instance(&[("alpha", 0.0)], DeviceKind::Varistor))
                .is_err()
        );
    }

    #[test]
    fn fuse_trips_above_trip_current() {
        let mut model = fuse(&[
            ("resistance", 0.1),
            ("trip_current", 1.0),
            ("trip_time", 1e-3),
        ]);
        stamp_fuse(&mut model);
        let mut solution = Solution::zero(2, 0);
        solution.node_voltages[1] = 1.0;
        for _ in 0..3 {
            model.update(&solution).expect("update");
        }
        let current = model.terminal_current(&solution).expect("current");
        assert!(current.abs() < 1e-9, "current={current}");
    }

    #[test]
    fn fuse_rejects_zero_trip_time() {
        assert!(
            FuseModel::from_instance(&instance(&[("trip_time", 0.0)], DeviceKind::Fuse)).is_err()
        );
    }
}
