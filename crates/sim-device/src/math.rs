//! Behavioral math blocks.

use crate::model::{DeviceModel, finite_parameter, pins_exact, positive_parameter, stamp_output};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

#[derive(Clone, Copy)]
pub(crate) enum MathKind {
    Multiplier,
    Divider,
    Logarithmic,
    Antilog,
    Rms,
}

/// A four-pin behavioral math block.
pub struct BehavioralMathModel {
    kind: MathKind,
    gain: f64,
    reference: f64,
    time_constant: f64,
    output_limit: f64,
    output_resistance: f64,
    mean_square: f64,
    dt: Option<f64>,
    pins: [NodeId; 4],
}

impl BehavioralMathModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        kind: MathKind,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        Ok(Self {
            kind,
            gain: finite_parameter(params, "gain", 1.0)?,
            reference: positive_parameter(params, "reference", Some(1.0))?,
            time_constant: positive_parameter(params, "time_constant", Some(1e-3))?,
            output_limit: positive_parameter(params, "output_limit", Some(15.0))?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            mean_square: 0.0,
            dt: None,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn compute(&self, first: f64, second: f64) -> f64 {
        let value = match self.kind {
            MathKind::Multiplier => self.gain * first * second,
            MathKind::Divider => {
                if second.abs() < 1e-9 {
                    self.output_limit.copysign(first)
                } else {
                    self.gain * first / second
                }
            }
            MathKind::Logarithmic => {
                if first <= 0.0 {
                    -self.output_limit
                } else {
                    self.gain * (first / self.reference).ln()
                }
            }
            MathKind::Antilog => {
                let exponent = (first / self.reference).min(50.0);
                self.gain * self.reference * exponent.exp()
            }
            MathKind::Rms => self.gain * self.mean_square.max(0.0).sqrt(),
        };
        value.clamp(-self.output_limit, self.output_limit)
    }

    fn advance(&mut self, first: f64, dt: f64) {
        if !matches!(self.kind, MathKind::Rms) {
            return;
        }
        let alpha = dt / (self.time_constant + dt);
        self.mean_square += (first * first - self.mean_square) * alpha;
    }
}

impl DeviceModel for BehavioralMathModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt = dt;
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let second = solution.voltage(self.pins[1]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let value = if matches!(self.kind, MathKind::Rms) && dt.is_none() {
            self.gain * first.abs()
        } else {
            self.compute(first, second)
        };
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[3],
            value,
            self.output_resistance,
        )
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(dt) = self.dt else {
            return Ok(());
        };
        let first = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        self.advance(first, dt);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralMathModel, MathKind};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = [("reference", 1.0)]
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
                NodeId::new(3),
                NodeId::GROUND,
            ],
            parameters,
        )
    }

    #[test]
    fn multiplier_scales_product() {
        let model = BehavioralMathModel::from_instance(
            &instance(DeviceKind::BehavioralMultiplier),
            MathKind::Multiplier,
        )
        .expect("model");
        assert!((model.compute(2.0, 3.0) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn divider_rejects_zero() {
        let model = BehavioralMathModel::from_instance(
            &instance(DeviceKind::BehavioralDivider),
            MathKind::Divider,
        )
        .expect("model");
        assert!((model.compute(4.0, 2.0) - 2.0).abs() < 1e-9);
        assert!(model.compute(4.0, 0.0).abs() > 1.0);
    }

    #[test]
    fn logarithmic_and_antilog_round_trip() {
        let log = BehavioralMathModel::from_instance(
            &instance(DeviceKind::BehavioralLog),
            MathKind::Logarithmic,
        )
        .expect("log");
        let antilog = BehavioralMathModel::from_instance(
            &instance(DeviceKind::BehavioralAntilog),
            MathKind::Antilog,
        )
        .expect("antilog");
        assert!((log.compute(2.0_f64.exp(), 0.0) - 2.0).abs() < 1e-9);
        assert!((antilog.compute(2.0_f64.ln(), 0.0) - 2.0).abs() < 1e-9);
    }

    #[test]
    fn rms_converges_to_dc_magnitude() {
        let mut model =
            BehavioralMathModel::from_instance(&instance(DeviceKind::BehavioralRms), MathKind::Rms)
                .expect("model");
        for _ in 0..100 {
            model.advance(3.0, 1e-3);
        }
        assert!((model.compute(0.0, 0.0) - 3.0).abs() < 0.01);
    }
}
