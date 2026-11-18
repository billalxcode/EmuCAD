//! Behavioral oscillator blocks.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_output,
};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

#[derive(Clone, Copy)]
pub(crate) enum OscillatorKind {
    Vco,
    Amplitude,
    Frequency,
    Pwm,
}

/// A four-pin behavioral oscillator block.
pub struct BehavioralOscillatorModel {
    kind: OscillatorKind,
    frequency: f64,
    sensitivity: f64,
    control_center: f64,
    amplitude: f64,
    offset: f64,
    output_resistance: f64,
    phase: f64,
    dt: Option<f64>,
    pins: [NodeId; 4],
}

impl BehavioralOscillatorModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        kind: OscillatorKind,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        let sensitivity = match kind {
            OscillatorKind::Vco | OscillatorKind::Frequency => 1e3,
            OscillatorKind::Pwm => 0.1,
            OscillatorKind::Amplitude => 1.0,
        };
        Ok(Self {
            kind,
            frequency: positive_parameter(params, "frequency", Some(1e3))?,
            sensitivity: non_negative_parameter(params, "sensitivity", sensitivity)?,
            control_center: finite_parameter(params, "control_center", 2.5)?,
            amplitude: finite_parameter(params, "amplitude", 1.0)?,
            offset: finite_parameter(params, "offset", 0.0)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            phase: 0.0,
            dt: None,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn duty(&self, control: f64) -> f64 {
        (0.5 + self.sensitivity * (control - self.control_center)).clamp(0.01, 0.99)
    }

    fn instantaneous_frequency(&self, control: f64) -> f64 {
        (self.frequency + self.sensitivity * (control - self.control_center)).max(0.0)
    }

    fn output(&self, control: f64, supply: f64) -> f64 {
        let value = match self.kind {
            OscillatorKind::Vco => {
                if self.phase < 0.5 {
                    supply.max(0.0)
                } else {
                    0.0
                }
            }
            OscillatorKind::Pwm => {
                if self.phase < self.duty(control) {
                    supply.max(0.0)
                } else {
                    0.0
                }
            }
            OscillatorKind::Amplitude => {
                let envelope = 1.0 + self.sensitivity * (control - self.control_center);
                let carrier = (2.0 * std::f64::consts::PI * self.phase).sin();
                self.offset + self.amplitude * envelope * carrier
            }
            OscillatorKind::Frequency => {
                let carrier = (2.0 * std::f64::consts::PI * self.phase).sin();
                self.offset + self.amplitude * carrier
            }
        };
        value.clamp(0.0, supply.max(0.0))
    }

    fn advance(&mut self, control: f64, dt: f64) {
        let frequency = match self.kind {
            OscillatorKind::Vco | OscillatorKind::Frequency => {
                self.instantaneous_frequency(control)
            }
            OscillatorKind::Amplitude | OscillatorKind::Pwm => self.frequency,
        };
        self.phase = (self.phase + frequency * dt).fract();
    }
}

impl DeviceModel for BehavioralOscillatorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt = dt;
        let control = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let supply = solution.voltage(self.pins[2]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let value = self.output(control, supply);
        stamp_output(
            ctx,
            self.pins[1],
            self.pins[3],
            value,
            self.output_resistance,
        )
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(dt) = self.dt else {
            return Ok(());
        };
        let control = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        self.advance(control, dt);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralOscillatorModel, OscillatorKind};
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = BTreeMap::new();
        DeviceInstance::new(
            DeviceId::new(0),
            "U1".to_string(),
            kind,
            vec![
                NodeId::new(1),
                NodeId::new(2),
                NodeId::new(3),
                NodeId::GROUND,
            ],
            parameters,
        )
    }

    #[test]
    fn vco_frequency_scales_with_control() {
        let model = BehavioralOscillatorModel::from_instance(
            &instance(DeviceKind::BehavioralVco),
            OscillatorKind::Vco,
        )
        .expect("model");
        assert!((model.instantaneous_frequency(2.5) - 1e3).abs() < 1e-9);
        assert!((model.instantaneous_frequency(3.5) - 2e3).abs() < 1e-9);
    }

    #[test]
    fn pwm_duty_scales_with_control() {
        let model = BehavioralOscillatorModel::from_instance(
            &instance(DeviceKind::BehavioralPwm),
            OscillatorKind::Pwm,
        )
        .expect("model");
        assert!((model.duty(2.5) - 0.5).abs() < 1e-9);
        assert!((model.duty(3.5) - 1.5).abs() > 0.4);
    }

    #[test]
    fn amplitude_oscillator_is_bounded() {
        let model = BehavioralOscillatorModel::from_instance(
            &instance(DeviceKind::BehavioralAmplitude),
            OscillatorKind::Amplitude,
        )
        .expect("model");
        let value = model.output(2.5, 5.0);
        assert!((0.0..=5.0).contains(&value));
    }
}
