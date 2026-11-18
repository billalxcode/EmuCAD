//! Piezoelectric crystal model using a BVD equivalent circuit.

use crate::model::{
    DeviceModel, IntegrationMethod, non_negative_parameter, pins_exact, positive_parameter,
    stamp_capacitor, stamp_conductance, stamp_current_source,
};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A two-terminal crystal with a series RLC motional branch and parallel C0.
pub struct CrystalModel {
    resistance: f64,
    inductance: f64,
    capacitance: f64,
    parallel: f64,
    current_prev: f64,
    voltage_prev: f64,
    vc_prev: f64,
    dt_last: f64,
    conductance: f64,
    source: f64,
    pins: [NodeId; 2],
}

impl CrystalModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let frequency = positive_parameter(instance.parameters(), "frequency", Some(1e6))?;
        let capacitance = positive_parameter(instance.parameters(), "capacitance", Some(1e-14))?;
        let resistance = positive_parameter(instance.parameters(), "resistance", Some(50.0))?;
        let parallel = non_negative_parameter(instance.parameters(), "c0", 1e-12)?;
        let inductance = series_inductance(frequency, capacitance)?;
        Ok(Self {
            resistance,
            inductance,
            capacitance,
            parallel,
            current_prev: 0.0,
            voltage_prev: 0.0,
            vc_prev: 0.0,
            dt_last: 0.0,
            conductance: 0.0,
            source: 0.0,
            pins: [pins[0], pins[1]],
        })
    }
}

impl CrystalModel {
    fn stamp_series(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        let series = self.resistance + self.inductance / dt + dt / self.capacitance;
        let conductance = 1.0 / series;
        let source = (self.inductance / dt * self.current_prev - self.vc_prev) * conductance;
        stamp_conductance(ctx, self.pins[0], self.pins[1], conductance)?;
        stamp_current_source(ctx, self.pins[0], self.pins[1], source)?;
        self.dt_last = dt;
        self.conductance = conductance;
        self.source = source;
        Ok(())
    }

    fn stamp_parallel(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        stamp_capacitor(
            ctx,
            self.pins[0],
            self.pins[1],
            self.parallel,
            self.voltage_prev,
            dt,
        )
    }
}

fn series_inductance(frequency: f64, capacitance: f64) -> Result<f64, SimError> {
    let omega = std::f64::consts::TAU * frequency;
    let inductance = 1.0 / (omega * omega * capacitance);
    if !inductance.is_finite() || inductance <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "crystal frequency is out of range".to_string(),
        ));
    }
    Ok(inductance)
}

impl DeviceModel for CrystalModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.stamp_method(ctx, solution, dt, IntegrationMethod::BackwardEuler)
    }

    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
        method: IntegrationMethod,
    ) -> Result<(), SimError> {
        let Some(dt) = dt else {
            return Ok(());
        };
        if method != IntegrationMethod::BackwardEuler {
            return Err(SimError::UnsupportedDevice(
                "crystal supports backward Euler only".to_string(),
            ));
        }
        self.stamp_series(ctx, dt)?;
        self.stamp_parallel(ctx, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        if self.dt_last <= 0.0 {
            return Ok(());
        }
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0);
        let voltage = first - second;
        let current = self.conductance * voltage + self.source;
        self.vc_prev += self.dt_last / self.capacitance * current;
        self.current_prev = current;
        self.voltage_prev = voltage;
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current_prev)
    }
}

#[cfg(test)]
mod tests {
    use super::{CrystalModel, series_inductance};
    use crate::model::DeviceModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "x1",
            DeviceKind::Crystal,
            vec![NodeId::new(1), NodeId::GROUND],
            parameters,
        )
    }

    #[test]
    fn series_resonance_frequency_is_one_megahertz() {
        let inductance = series_inductance(1e6, 1e-14).expect("inductance");
        let frequency = 1.0 / (std::f64::consts::TAU * (inductance * 1e-14).sqrt());
        assert!((frequency - 1e6).abs() < 1.0, "frequency={frequency}");
    }

    #[test]
    fn crystal_is_open_at_dc() {
        let mut model = CrystalModel::from_instance(&instance(&[])).expect("model");
        let mut ctx = kernel::MnaContext::new(1, 0, vec![None, Some(0)], vec![]).expect("ctx");
        model
            .stamp(&mut ctx, &Solution::zero(2, 1), None)
            .expect("stamp");
    }
}
