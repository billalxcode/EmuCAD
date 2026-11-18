//! Behavioral timer modules.

use crate::model::{
    DeviceModel, finite_parameter, non_negative_parameter, pins_exact, positive_parameter,
    stamp_output,
};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

const TIMER_PINS: usize = 8;
const OUT: usize = 2;
const TRIGGER: usize = 4;
const CONTROL: usize = 5;

fn timer_pins(instance: &DeviceInstance) -> Result<[NodeId; TIMER_PINS], SimError> {
    let pins = pins_exact(instance, TIMER_PINS)?;
    let mut array = [NodeId::GROUND; TIMER_PINS];
    array.copy_from_slice(&pins);
    Ok(array)
}

fn timer_supply(solution: &Solution, pins: [NodeId; TIMER_PINS]) -> f64 {
    solution.voltage(pins[0]).unwrap_or(0.0) - solution.voltage(pins[1]).unwrap_or(0.0)
}

fn duty_value(params: &std::collections::BTreeMap<String, f64>) -> Result<f64, SimError> {
    let duty = non_negative_parameter(params, "duty", 0.5)?;
    if !(0.0..1.0).contains(&duty) {
        return Err(SimError::InvalidNetlist(
            "timer duty must be between zero and one".to_string(),
        ));
    }
    Ok(duty)
}

fn stamp_timer_output(
    ctx: &mut MnaContext,
    pins: [NodeId; TIMER_PINS],
    value: f64,
    resistance: f64,
) -> Result<(), SimError> {
    stamp_output(ctx, pins[OUT], pins[1], value, resistance)
}

/// An astable timer with a fixed period and duty cycle.
pub struct AstableTimerModel {
    period: f64,
    duty: f64,
    output_resistance: f64,
    time: f64,
    step: Option<f64>,
    pins: [NodeId; TIMER_PINS],
}

impl AstableTimerModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            period: positive_parameter(params, "period", Some(1e-3))?,
            duty: duty_value(params)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            time: 0.0,
            step: None,
            pins: timer_pins(instance)?,
        })
    }

    fn value_at(&self, time: f64, supply: f64) -> f64 {
        let phase = (time % self.period) / self.period;
        if phase < self.duty {
            supply.max(0.0)
        } else {
            0.0
        }
    }
}

impl DeviceModel for AstableTimerModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        let supply = timer_supply(solution, self.pins);
        let value = self.value_at(self.time, supply);
        stamp_timer_output(ctx, self.pins, value, self.output_resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }
}

/// A monostable timer triggered by a rising edge on pin 4.
pub struct MonostableTimerModel {
    period: f64,
    trigger_threshold: f64,
    output_resistance: f64,
    trigger_high: bool,
    remaining: f64,
    step: Option<f64>,
    pins: [NodeId; TIMER_PINS],
}

impl MonostableTimerModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            period: positive_parameter(params, "period", Some(1e-3))?,
            trigger_threshold: finite_parameter(params, "trigger_threshold", 2.5)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            trigger_high: false,
            remaining: 0.0,
            step: None,
            pins: timer_pins(instance)?,
        })
    }

    fn trigger(&mut self, trigger: f64) {
        let high = trigger > self.trigger_threshold;
        if high && !self.trigger_high {
            self.remaining = self.period;
        }
        self.trigger_high = high;
    }
}

impl DeviceModel for MonostableTimerModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
            }
        }
        let trigger = solution.voltage(self.pins[TRIGGER]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        self.trigger(trigger);
        let supply = timer_supply(solution, self.pins);
        let value = if self.remaining > 0.0 {
            supply.max(0.0)
        } else {
            0.0
        };
        stamp_timer_output(ctx, self.pins, value, self.output_resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.remaining = (self.remaining - dt).max(0.0);
        }
        Ok(())
    }
}

/// A voltage-controlled oscillator with a square-wave output.
pub struct VcoTimerModel {
    center_period: f64,
    sensitivity: f64,
    control_center: f64,
    output_resistance: f64,
    phase: f64,
    step: Option<f64>,
    pins: [NodeId; TIMER_PINS],
}

impl VcoTimerModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            center_period: positive_parameter(params, "period", Some(1e-3))?,
            sensitivity: non_negative_parameter(params, "sensitivity", 1000.0)?,
            control_center: finite_parameter(params, "control_center", 2.5)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            phase: 0.0,
            step: None,
            pins: timer_pins(instance)?,
        })
    }

    fn frequency(&self, control: f64) -> f64 {
        (1.0 / self.center_period + self.sensitivity * (control - self.control_center)).max(0.0)
    }
}

impl DeviceModel for VcoTimerModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
            }
        }
        let supply = timer_supply(solution, self.pins);
        let value = if self.phase < 0.5 {
            supply.max(0.0)
        } else {
            0.0
        };
        stamp_timer_output(ctx, self.pins, value, self.output_resistance)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(dt) = self.step else {
            return Ok(());
        };
        let control = solution.voltage(self.pins[CONTROL]).unwrap_or(0.0)
            - solution.voltage(self.pins[1]).unwrap_or(0.0);
        let frequency = self.frequency(control);
        self.phase = (self.phase + frequency * dt).fract();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AstableTimerModel, MonostableTimerModel, VcoTimerModel};
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        let nodes = (0..8).map(|_| NodeId::GROUND).collect();
        DeviceInstance::new(DeviceId::new(0), "U1".to_string(), kind, nodes, parameters)
    }

    #[test]
    fn astable_output_toggles_with_duty() {
        let model = AstableTimerModel::from_instance(&instance(
            DeviceKind::AstableTimer,
            &[("period", 1e-3), ("duty", 0.25)],
        ))
        .expect("model");
        assert!((model.value_at(0.0, 5.0) - 5.0).abs() < 1e-9);
        assert!(model.value_at(0.5e-3, 5.0).abs() < 1e-9);
    }

    #[test]
    fn monostable_starts_on_rising_trigger() {
        let mut model = MonostableTimerModel::from_instance(&instance(
            DeviceKind::MonostableTimer,
            &[("period", 1e-3)],
        ))
        .expect("model");
        model.trigger(0.0);
        assert!(model.remaining.abs() < 1e-9);
        model.trigger(5.0);
        assert!((model.remaining - 1e-3).abs() < 1e-12);
        model.trigger(5.0);
        assert!((model.remaining - 1e-3).abs() < 1e-12);
    }

    #[test]
    fn vco_frequency_scales_with_control() {
        let model = VcoTimerModel::from_instance(&instance(
            DeviceKind::VcoTimer,
            &[("period", 1e-3), ("sensitivity", 1000.0)],
        ))
        .expect("model");
        assert!((model.frequency(2.5) - 1000.0).abs() < 1e-9);
        assert!((model.frequency(3.5) - 2000.0).abs() < 1e-9);
    }
}
