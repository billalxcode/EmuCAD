//! Behavioral filter blocks.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_output,
};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

#[derive(Clone, Copy)]
pub(crate) enum FilterKind {
    Integrator,
    Differentiator,
    LowPass,
    HighPass,
    StateVariable,
}

/// A four-pin behavioral filter block.
pub struct BehavioralFilterModel {
    kind: FilterKind,
    gain: f64,
    time_constant: f64,
    damping: f64,
    output_limit: f64,
    output_resistance: f64,
    state: f64,
    aux_state: f64,
    previous_input: f64,
    dt: Option<f64>,
    pins: [NodeId; 4],
}

impl BehavioralFilterModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        kind: FilterKind,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        Ok(Self {
            kind,
            gain: finite_parameter(params, "gain", 1.0)?,
            time_constant: positive_parameter(params, "time_constant", Some(1e-3))?,
            damping: non_negative_parameter(params, "damping", 0.7)?,
            output_limit: positive_parameter(params, "output_limit", Some(15.0))?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            state: 0.0,
            aux_state: 0.0,
            previous_input: 0.0,
            dt: None,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn clamp_state(&self, value: f64) -> f64 {
        value.clamp(-self.output_limit, self.output_limit)
    }

    fn dc_output(&self, input: f64) -> f64 {
        match self.kind {
            FilterKind::LowPass | FilterKind::StateVariable => self.gain * input,
            _ => 0.0,
        }
    }

    fn transient_output(&self) -> f64 {
        match self.kind {
            FilterKind::Integrator
            | FilterKind::LowPass
            | FilterKind::HighPass
            | FilterKind::StateVariable => self.gain * self.state,
            FilterKind::Differentiator => self.gain * self.time_constant * self.state,
        }
    }

    fn advance(&mut self, input: f64, dt: f64) {
        let tau = self.time_constant;
        match self.kind {
            FilterKind::Integrator => {
                self.state = self.clamp_state(self.state + input * dt / tau);
            }
            FilterKind::Differentiator => {
                self.state = (input - self.previous_input) / dt;
            }
            FilterKind::LowPass => {
                let alpha = dt / (tau + dt);
                self.state = self.clamp_state(self.state + (input - self.state) * alpha);
            }
            FilterKind::HighPass => {
                let alpha = dt / (tau + dt);
                self.aux_state = self.aux_state + (input - self.aux_state) * alpha;
                self.state = input - self.aux_state;
            }
            FilterKind::StateVariable => {
                let omega = dt / tau;
                let low = self.state + self.aux_state * omega;
                let band =
                    self.aux_state + (input - self.state - self.damping * self.aux_state) * omega;
                self.state = self.clamp_state(low);
                self.aux_state = self.clamp_state(band);
            }
        }
        self.previous_input = input;
    }

    fn aux_output(&self) -> Option<f64> {
        match self.kind {
            FilterKind::StateVariable => Some(self.gain * self.aux_state),
            _ => None,
        }
    }
}

impl DeviceModel for BehavioralFilterModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt = dt;
        let input = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[2]).unwrap_or(0.0);
        let value = if dt.is_some() {
            self.transient_output()
        } else {
            self.dc_output(input)
        };
        stamp_output(
            ctx,
            self.pins[1],
            self.pins[2],
            value,
            self.output_resistance,
        )?;
        if let Some(aux) = self.aux_output() {
            stamp_output(ctx, self.pins[3], self.pins[2], aux, self.output_resistance)?;
        }
        Ok(())
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(dt) = self.dt else {
            return Ok(());
        };
        let input = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[2]).unwrap_or(0.0);
        self.advance(input, dt);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralFilterModel, FilterKind};
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = [("time_constant", 1e-3)]
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "U1".to_string(),
            kind,
            vec![
                NodeId::new(1),
                NodeId::new(2),
                NodeId::GROUND,
                NodeId::new(3),
            ],
            parameters,
        )
    }

    #[test]
    fn integrator_accumulates_input() {
        let mut model = BehavioralFilterModel::from_instance(
            &instance(DeviceKind::BehavioralIntegrator),
            FilterKind::Integrator,
        )
        .expect("model");
        model.advance(1.0, 1e-4);
        assert!((model.state - 0.1).abs() < 1e-9);
        model.advance(1.0, 1e-4);
        assert!((model.state - 0.2).abs() < 1e-9);
    }

    #[test]
    fn low_pass_approaches_input() {
        let mut model = BehavioralFilterModel::from_instance(
            &instance(DeviceKind::BehavioralLowPass),
            FilterKind::LowPass,
        )
        .expect("model");
        for _ in 0..10 {
            model.advance(1.0, 1e-4);
        }
        assert!(model.state > 0.5);
    }

    #[test]
    fn high_pass_rejects_dc() {
        let mut model = BehavioralFilterModel::from_instance(
            &instance(DeviceKind::BehavioralHighPass),
            FilterKind::HighPass,
        )
        .expect("model");
        for _ in 0..100 {
            model.advance(1.0, 1e-4);
        }
        assert!(model.state.abs() < 1e-3);
    }
}
