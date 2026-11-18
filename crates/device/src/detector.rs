//! Behavioral detector blocks.

use crate::model::{DeviceModel, finite_parameter, pins_exact, positive_parameter, stamp_output};
use kernel::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

#[derive(Clone, Copy)]
pub(crate) enum DetectorKind {
    Comparator,
    Schmitt,
    Peak,
    TrackHold,
    Rectifier,
}

/// A four-pin behavioral detector block.
pub struct BehavioralDetectorModel {
    kind: DetectorKind,
    gain: f64,
    threshold: f64,
    upper_threshold: f64,
    lower_threshold: f64,
    reset_threshold: f64,
    output_high: f64,
    output_low: f64,
    output_limit: f64,
    output_resistance: f64,
    state: f64,
    on: bool,
    pins: [NodeId; 4],
}

impl BehavioralDetectorModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        kind: DetectorKind,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        Ok(Self {
            kind,
            gain: finite_parameter(params, "gain", 1.0)?,
            threshold: finite_parameter(params, "threshold", 2.5)?,
            upper_threshold: finite_parameter(params, "upper_threshold", 3.0)?,
            lower_threshold: finite_parameter(params, "lower_threshold", 2.0)?,
            reset_threshold: finite_parameter(params, "reset_threshold", 2.5)?,
            output_high: finite_parameter(params, "output_high", 5.0)?,
            output_low: finite_parameter(params, "output_low", 0.0)?,
            output_limit: positive_parameter(params, "output_limit", Some(15.0))?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            state: 0.0,
            on: false,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn update_state(&mut self, input: f64, reference: f64) {
        match self.kind {
            DetectorKind::Schmitt => {
                if input > self.upper_threshold {
                    self.on = true;
                }
                if input < self.lower_threshold {
                    self.on = false;
                }
            }
            DetectorKind::Peak => {
                if reference > self.reset_threshold {
                    self.state = 0.0;
                } else if input > self.state {
                    self.state = input;
                }
            }
            DetectorKind::TrackHold if reference > self.threshold => {
                self.state = input;
            }
            _ => {}
        }
    }

    fn output(&self, input: f64, reference: f64) -> f64 {
        match self.kind {
            DetectorKind::Comparator => {
                if input > reference {
                    self.output_high
                } else {
                    self.output_low
                }
            }
            DetectorKind::Schmitt => {
                if self.on {
                    self.output_high
                } else {
                    self.output_low
                }
            }
            DetectorKind::Peak | DetectorKind::TrackHold => {
                (self.gain * self.state).clamp(-self.output_limit, self.output_limit)
            }
            DetectorKind::Rectifier => {
                (self.gain * input.max(0.0)).clamp(-self.output_limit, self.output_limit)
            }
        }
    }
}

impl DeviceModel for BehavioralDetectorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let input = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let reference = solution.voltage(self.pins[1]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        self.update_state(input, reference);
        let value = self.output(input, reference);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[3],
            value,
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralDetectorModel, DetectorKind};
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
    fn comparator_uses_reference_pin() {
        let model = BehavioralDetectorModel::from_instance(
            &instance(DeviceKind::BehavioralComparator),
            DetectorKind::Comparator,
        )
        .expect("model");
        assert!((model.output(1.0, 0.5) - 5.0).abs() < 1e-9);
        assert!(model.output(0.2, 0.5).abs() < 1e-9);
    }

    #[test]
    fn schmitt_uses_upper_and_lower_thresholds() {
        let mut model = BehavioralDetectorModel::from_instance(
            &instance(DeviceKind::BehavioralSchmitt),
            DetectorKind::Schmitt,
        )
        .expect("model");
        model.update_state(3.5, 0.0);
        assert!(model.on);
        model.update_state(2.5, 0.0);
        assert!(model.on);
        model.update_state(1.5, 0.0);
        assert!(!model.on);
    }

    #[test]
    fn rectifier_clips_negative_input() {
        let model = BehavioralDetectorModel::from_instance(
            &instance(DeviceKind::BehavioralPrecisionRectifier),
            DetectorKind::Rectifier,
        )
        .expect("model");
        assert!(model.output(-2.0, 0.0).abs() < 1e-9);
        assert!((model.output(2.0, 0.0) - 2.0).abs() < 1e-9);
    }
}
