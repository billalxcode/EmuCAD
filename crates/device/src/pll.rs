//! Behavioral phase-locked loop modules.

use crate::model::{DeviceModel, finite_parameter, pins_exact, positive_parameter, stamp_output};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A behavioral PLL implemented as a phase-reset frequency multiplier.
pub struct PllModel {
    multiplier: f64,
    free_frequency: f64,
    threshold: f64,
    output_resistance: f64,
    phase: f64,
    time: f64,
    last_edge: Option<f64>,
    input_high: bool,
    dt: Option<f64>,
    input: NodeId,
    output: NodeId,
    supply: NodeId,
    ground: NodeId,
}

impl PllModel {
    pub(crate) fn from_timer(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 8)?;
        Self::new(instance, [pins[4], pins[2], pins[0], pins[1]])
    }

    pub(crate) fn from_behavioral(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        Self::new(instance, [pins[0], pins[1], pins[2], pins[3]])
    }

    fn new(instance: &DeviceInstance, pins: [NodeId; 4]) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            multiplier: positive_parameter(params, "multiplier", Some(2.0))?,
            free_frequency: positive_parameter(params, "frequency", Some(1e3))?,
            threshold: finite_parameter(params, "threshold", 2.5)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            phase: 0.0,
            time: 0.0,
            last_edge: None,
            input_high: false,
            dt: None,
            input: pins[0],
            output: pins[1],
            supply: pins[2],
            ground: pins[3],
        })
    }

    fn output_frequency(&self) -> f64 {
        match self.last_edge {
            Some(edge) if self.time > edge => self.multiplier / (self.time - edge),
            _ => self.free_frequency,
        }
    }

    fn output(&self, supply: f64) -> f64 {
        if self.phase < 0.5 {
            supply.max(0.0)
        } else {
            0.0
        }
    }
}

impl DeviceModel for PllModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt = dt;
        let supply = solution.voltage(self.supply).unwrap_or(0.0)
            - solution.voltage(self.ground).unwrap_or(0.0);
        let value = self.output(supply);
        stamp_output(ctx, self.output, self.ground, value, self.output_resistance)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(dt) = self.dt else {
            return Ok(());
        };
        self.time += dt;
        let input = solution.voltage(self.input).unwrap_or(0.0)
            - solution.voltage(self.ground).unwrap_or(0.0);
        let high = input > self.threshold;
        if high && !self.input_high {
            self.last_edge = Some(self.time);
            self.phase = 0.0;
        }
        self.input_high = high;
        self.phase = (self.phase + self.output_frequency() * dt).fract();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PllModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, pins: usize) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = BTreeMap::new();
        let nodes = (0..pins).map(|_| NodeId::GROUND).collect();
        DeviceInstance::new(DeviceId::new(0), "U1".to_string(), kind, nodes, parameters)
    }

    #[test]
    fn timer_pll_builds_from_eight_pins() {
        assert!(PllModel::from_timer(&instance(DeviceKind::TimerPll, 8)).is_ok());
    }

    #[test]
    fn behavioral_pll_builds_from_four_pins() {
        assert!(PllModel::from_behavioral(&instance(DeviceKind::BehavioralPll, 4)).is_ok());
    }

    #[test]
    fn pll_frequency_multiplies_reference() {
        let mut model =
            PllModel::from_behavioral(&instance(DeviceKind::BehavioralPll, 4)).expect("model");
        model.last_edge = Some(0.0);
        model.time = 1e-3;
        assert!((model.output_frequency() - 2000.0).abs() < 1e-9);
    }
}
