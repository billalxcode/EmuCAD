//! Behavioral converter blocks.

use crate::model::{DeviceModel, finite_parameter, pins_exact, positive_parameter, stamp_output};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

#[derive(Clone, Copy)]
pub(crate) enum ConverterBlockKind {
    SampleHold,
    Sar,
    DeltaSigma,
    PwmDac,
}

/// A four-pin behavioral converter or sampler.
pub struct BehavioralConverterModel {
    kind: ConverterBlockKind,
    gain: f64,
    threshold: f64,
    resolution: u32,
    full_scale: f64,
    time_constant: f64,
    output_limit: f64,
    output_resistance: f64,
    state: f64,
    clock_high: bool,
    dt: Option<f64>,
    pins: [NodeId; 4],
}

impl BehavioralConverterModel {
    pub(crate) fn from_instance(
        instance: &DeviceInstance,
        kind: ConverterBlockKind,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        let resolution = finite_parameter(params, "resolution", 8.0)?;
        Ok(Self {
            kind,
            gain: finite_parameter(params, "gain", 1.0)?,
            threshold: finite_parameter(params, "threshold", 2.5)?,
            resolution: (resolution.round() as u32).clamp(1, 16),
            full_scale: positive_parameter(params, "full_scale", Some(5.0))?,
            time_constant: positive_parameter(params, "time_constant", Some(1e-3))?,
            output_limit: positive_parameter(params, "output_limit", Some(15.0))?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            state: 0.0,
            clock_high: false,
            dt: None,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }

    fn quantize(&self, value: f64) -> f64 {
        let levels = ((1u32 << self.resolution) - 1).max(1) as f64;
        let code = (value / self.full_scale * levels)
            .round()
            .clamp(0.0, levels);
        code * self.full_scale / levels
    }

    fn sample(&mut self, input: f64) {
        match self.kind {
            ConverterBlockKind::SampleHold => self.state = input,
            ConverterBlockKind::Sar => self.state = self.quantize(input),
            ConverterBlockKind::DeltaSigma => {
                let filtered = self.state + (input - self.state) * 0.1;
                self.state = self.quantize(filtered);
            }
            ConverterBlockKind::PwmDac => {}
        }
    }

    fn output(&self) -> f64 {
        (self.gain * self.state).clamp(-self.output_limit, self.output_limit)
    }
}

impl DeviceModel for BehavioralConverterModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.dt = dt;
        let input = solution.voltage(self.pins[0]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        let control = solution.voltage(self.pins[1]).unwrap_or(0.0)
            - solution.voltage(self.pins[3]).unwrap_or(0.0);
        match self.kind {
            ConverterBlockKind::SampleHold => {
                if control > self.threshold {
                    self.state = input;
                }
            }
            ConverterBlockKind::Sar | ConverterBlockKind::DeltaSigma => {
                let high = control > self.threshold;
                if dt.is_none() || (high && !self.clock_high) {
                    self.sample(input);
                }
                self.clock_high = high;
            }
            ConverterBlockKind::PwmDac => {
                if let Some(dt) = dt {
                    let alpha = dt / (self.time_constant + dt);
                    self.state += (input - self.state) * alpha;
                } else {
                    self.state = input;
                }
            }
        }
        let output = if matches!(self.kind, ConverterBlockKind::PwmDac) {
            self.pins[1]
        } else {
            self.pins[2]
        };
        stamp_output(
            ctx,
            output,
            self.pins[3],
            self.output(),
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BehavioralConverterModel, ConverterBlockKind};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
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
    fn sar_quantizes_to_resolution() {
        let model = BehavioralConverterModel::from_instance(
            &instance(
                DeviceKind::BehavioralSarAdc,
                &[("resolution", 8.0), ("full_scale", 5.0)],
            ),
            ConverterBlockKind::Sar,
        )
        .expect("model");
        assert!((model.quantize(2.5) - 2.5098039215686274).abs() < 1e-9);
        assert!(model.quantize(-1.0).abs() < 1e-9);
        assert!((model.quantize(9.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn pwm_dac_builds() {
        assert!(
            BehavioralConverterModel::from_instance(
                &instance(DeviceKind::BehavioralPwmDac, &[]),
                ConverterBlockKind::PwmDac,
            )
            .is_ok()
        );
    }
}
