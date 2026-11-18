//! Independent source models.

use crate::model::{
    DeviceModel, branch_index, finite_parameter, non_negative_parameter, parameter, pins_exact,
    positive_parameter, stamp_current_source, stamp_voltage_branch,
};
use kernel::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// An independent voltage source.
pub struct VoltageSourceModel {
    id: DeviceId,
    voltage: f64,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl VoltageSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let voltage = parameter(instance.parameters(), "voltage", 0.0);
        Ok(Self {
            id: instance.id(),
            voltage,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for VoltageSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.branch = Some(branch_index(ctx, self.id)?);
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], self.voltage)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }

    fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), SimError> {
        if name != "voltage" {
            return Err(SimError::InvalidNetlist(format!(
                "voltage source parameter {name} is not writable"
            )));
        }
        self.voltage = value;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum PulseMode {
    Voltage,
    Current,
}

/// An independent pulse voltage or current source.
pub struct PulseSourceModel {
    id: DeviceId,
    mode: PulseMode,
    initial: f64,
    pulsed: f64,
    timing: PulseTiming,
    time: f64,
    step: Option<f64>,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

#[derive(Clone, Copy)]
struct PulseTiming {
    delay: f64,
    rise: f64,
    fall: f64,
    width: f64,
    period: f64,
}

impl PulseSourceModel {
    pub(crate) fn voltage(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, PulseMode::Voltage)
    }

    pub(crate) fn current(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, PulseMode::Current)
    }

    fn from_instance(instance: &DeviceInstance, mode: PulseMode) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let default_pulsed = match mode {
            PulseMode::Voltage => 5.0,
            PulseMode::Current => 1e-3,
        };
        Ok(Self {
            id: instance.id(),
            mode,
            initial: parameter(params, "v1", 0.0),
            pulsed: parameter(params, "v2", default_pulsed),
            timing: pulse_timing(params)?,
            time: 0.0,
            step: None,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        if time < self.timing.delay {
            return self.initial;
        }
        let phase = (time - self.timing.delay) % self.timing.period;
        if self.timing.rise > 0.0 && phase < self.timing.rise {
            return self.initial + (self.pulsed - self.initial) * phase / self.timing.rise;
        }
        let fall_start = self.timing.rise + self.timing.width;
        if phase < fall_start {
            return self.pulsed;
        }
        if self.timing.fall > 0.0 && phase < fall_start + self.timing.fall {
            let ratio = (phase - fall_start) / self.timing.fall;
            return self.pulsed + (self.initial - self.pulsed) * ratio;
        }
        self.initial
    }
}

fn pulse_timing(params: &std::collections::BTreeMap<String, f64>) -> Result<PulseTiming, SimError> {
    Ok(PulseTiming {
        delay: non_negative_parameter(params, "delay", 0.0)?,
        rise: non_negative_parameter(params, "rise", 1e-9)?,
        fall: non_negative_parameter(params, "fall", 1e-9)?,
        width: non_negative_parameter(params, "width", 1e-6)?,
        period: positive_parameter(params, "period", Some(2e-6))?,
    })
}

impl DeviceModel for PulseSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        let value = self.value_at(self.time);
        match self.mode {
            PulseMode::Voltage => {
                self.branch = Some(branch_index(ctx, self.id)?);
                stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], value)
            }
            PulseMode::Current => stamp_current_source(ctx, self.pins[0], self.pins[1], value),
        }
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        match self.mode {
            PulseMode::Voltage => solution.branch_current(self.branch?),
            PulseMode::Current => Some(self.value_at(self.time)),
        }
    }
}

#[derive(Clone, Copy)]
enum SineMode {
    Voltage,
    Current,
}

/// A sine voltage or current source.
pub struct SineSourceModel {
    id: DeviceId,
    mode: SineMode,
    amplitude: f64,
    frequency: f64,
    phase: f64,
    offset: f64,
    time: f64,
    step: Option<f64>,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl SineSourceModel {
    pub(crate) fn voltage(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, SineMode::Voltage)
    }

    pub(crate) fn current(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, SineMode::Current)
    }

    fn from_instance(instance: &DeviceInstance, mode: SineMode) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let default_amplitude = match mode {
            SineMode::Voltage => 5.0,
            SineMode::Current => 1e-3,
        };
        Ok(Self {
            id: instance.id(),
            mode,
            amplitude: finite_parameter(params, "amplitude", default_amplitude)?,
            frequency: non_negative_parameter(params, "frequency", 1e3)?,
            phase: finite_parameter(params, "phase", 0.0)?,
            offset: finite_parameter(params, "offset", 0.0)?,
            time: 0.0,
            step: None,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        let angle = 2.0 * std::f64::consts::PI * self.frequency * time + self.phase.to_radians();
        self.offset + self.amplitude * angle.sin()
    }
}

impl DeviceModel for SineSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        let value = self.value_at(self.time);
        match self.mode {
            SineMode::Voltage => {
                self.branch = Some(branch_index(ctx, self.id)?);
                stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], value)
            }
            SineMode::Current => stamp_current_source(ctx, self.pins[0], self.pins[1], value),
        }
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        match self.mode {
            SineMode::Voltage => solution.branch_current(self.branch?),
            SineMode::Current => Some(self.value_at(self.time)),
        }
    }
}

/// An exponential voltage source with two timed exponentials.
pub struct ExponentialSourceModel {
    id: DeviceId,
    initial: f64,
    pulsed: f64,
    delay1: f64,
    tau1: f64,
    delay2: f64,
    tau2: f64,
    time: f64,
    step: Option<f64>,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl ExponentialSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let delay1 = non_negative_parameter(params, "td1", 0.0)?;
        let delay2 = non_negative_parameter(params, "td2", 1e-3)?;
        if delay2 < delay1 {
            return Err(SimError::InvalidNetlist(
                "exponential source td2 must not precede td1".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            initial: finite_parameter(params, "v1", 0.0)?,
            pulsed: finite_parameter(params, "v2", 5.0)?,
            delay1,
            tau1: positive_parameter(params, "tau1", Some(1e-3))?,
            delay2,
            tau2: positive_parameter(params, "tau2", Some(1e-3))?,
            time: 0.0,
            step: None,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        if time < self.delay1 {
            return self.initial;
        }
        if time < self.delay2 {
            let ratio = 1.0 - (-(time - self.delay1) / self.tau1).exp();
            return self.initial + (self.pulsed - self.initial) * ratio;
        }
        let ratio1 = 1.0 - (-(self.delay2 - self.delay1) / self.tau1).exp();
        let peak = self.initial + (self.pulsed - self.initial) * ratio1;
        let ratio2 = 1.0 - (-(time - self.delay2) / self.tau2).exp();
        peak + (self.initial - self.pulsed) * ratio2
    }
}

impl DeviceModel for ExponentialSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        self.branch = Some(branch_index(ctx, self.id)?);
        let value = self.value_at(self.time);
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], value)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }
}

#[derive(Clone, Copy)]
struct PwlPoint {
    time: f64,
    value: f64,
}

/// A piecewise-linear voltage source with up to four points.
pub struct PwlSourceModel {
    id: DeviceId,
    points: [PwlPoint; 4],
    time: f64,
    step: Option<f64>,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl PwlSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let points = [
            PwlPoint {
                time: finite_parameter(params, "t0", 0.0)?,
                value: finite_parameter(params, "v0", 0.0)?,
            },
            PwlPoint {
                time: finite_parameter(params, "t1", 1e-3)?,
                value: finite_parameter(params, "v1", 5.0)?,
            },
            PwlPoint {
                time: finite_parameter(params, "t2", 2e-3)?,
                value: finite_parameter(params, "v2", 5.0)?,
            },
            PwlPoint {
                time: finite_parameter(params, "t3", 3e-3)?,
                value: finite_parameter(params, "v3", 0.0)?,
            },
        ];
        if points.windows(2).any(|pair| pair[1].time < pair[0].time) {
            return Err(SimError::InvalidNetlist(
                "PWL source times must be non-decreasing".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            points,
            time: 0.0,
            step: None,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        if time <= self.points[0].time {
            return self.points[0].value;
        }
        for index in 1..self.points.len() {
            let point = self.points[index];
            if time > point.time {
                continue;
            }
            let previous = self.points[index - 1];
            let span = point.time - previous.time;
            if span <= 0.0 {
                return point.value;
            }
            let ratio = (time - previous.time) / span;
            return previous.value + (point.value - previous.value) * ratio;
        }
        self.points[self.points.len() - 1].value
    }
}

impl DeviceModel for PwlSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        self.branch = Some(branch_index(ctx, self.id)?);
        let value = self.value_at(self.time);
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], value)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }
}

/// A time polynomial voltage source.
pub struct PolynomialSourceModel {
    id: DeviceId,
    coefficients: [f64; 4],
    time: f64,
    step: Option<f64>,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl PolynomialSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        Ok(Self {
            id: instance.id(),
            coefficients: [
                finite_parameter(params, "c0", 0.0)?,
                finite_parameter(params, "c1", 1.0)?,
                finite_parameter(params, "c2", 0.0)?,
                finite_parameter(params, "c3", 0.0)?,
            ],
            time: 0.0,
            step: None,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn value_at(&self, time: f64) -> f64 {
        let mut value = 0.0;
        let mut power = 1.0;
        for coefficient in self.coefficients {
            value += coefficient * power;
            power *= time;
        }
        value
    }
}

impl DeviceModel for PolynomialSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if let Some(dt) = dt {
            if self.step.is_none() {
                self.step = Some(dt);
                self.time = dt;
            }
        }
        self.branch = Some(branch_index(ctx, self.id)?);
        let value = self.value_at(self.time);
        stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], value)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if let Some(dt) = self.step {
            self.time += dt;
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }
}

#[derive(Clone, Copy)]
enum NoiseMode {
    Voltage,
    Current,
}

/// A deterministic pseudo-random voltage or current source.
pub struct NoiseSourceModel {
    id: DeviceId,
    mode: NoiseMode,
    amplitude: f64,
    state: u64,
    value: f64,
    stepped: bool,
    branch: Option<usize>,
    pins: [NodeId; 2],
}

impl NoiseSourceModel {
    pub(crate) fn voltage(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, NoiseMode::Voltage)
    }

    pub(crate) fn current(instance: &DeviceInstance) -> Result<Self, SimError> {
        Self::from_instance(instance, NoiseMode::Current)
    }

    fn from_instance(instance: &DeviceInstance, mode: NoiseMode) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let params = instance.parameters();
        let default_amplitude = match mode {
            NoiseMode::Voltage => 1e-3,
            NoiseMode::Current => 1e-6,
        };
        let seed = finite_parameter(params, "seed", 1.0)? as u64 | 1;
        Ok(Self {
            id: instance.id(),
            mode,
            amplitude: non_negative_parameter(params, "amplitude", default_amplitude)?,
            state: seed,
            value: 0.0,
            stepped: false,
            branch: None,
            pins: [pins[0], pins[1]],
        })
    }

    fn generate(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        let unit = (self.state >> 11) as f64 / (1u64 << 53) as f64;
        self.amplitude * (2.0 * unit - 1.0)
    }
}

impl DeviceModel for NoiseSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        if dt.is_some() && !self.stepped {
            self.stepped = true;
            self.value = self.generate();
        }
        match self.mode {
            NoiseMode::Voltage => {
                self.branch = Some(branch_index(ctx, self.id)?);
                stamp_voltage_branch(ctx, self.id, self.pins[0], self.pins[1], self.value)
            }
            NoiseMode::Current => stamp_current_source(ctx, self.pins[0], self.pins[1], self.value),
        }
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        if self.stepped {
            self.value = self.generate();
        }
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        match self.mode {
            NoiseMode::Voltage => solution.branch_current(self.branch?),
            NoiseMode::Current => Some(self.value),
        }
    }
}

/// An independent current source.
pub struct CurrentSourceModel {
    current: f64,
    pins: [NodeId; 2],
}

impl CurrentSourceModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 2)?;
        let current = parameter(instance.parameters(), "current", 0.0);
        Ok(Self {
            current,
            pins: [pins[0], pins[1]],
        })
    }
}

impl DeviceModel for CurrentSourceModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_current_source(ctx, self.pins[0], self.pins[1], self.current)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }

    fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), SimError> {
        if name != "current" {
            return Err(SimError::InvalidNetlist(format!(
                "current source parameter {name} is not writable"
            )));
        }
        self.current = value;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExponentialSourceModel, NoiseMode, NoiseSourceModel, PolynomialSourceModel, PulseMode,
        PulseSourceModel, PulseTiming, PwlSourceModel, SineMode, SineSourceModel,
    };
    use crate::model::DeviceModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};
    use std::collections::BTreeMap;

    fn source_instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "V1".to_string(),
            kind,
            vec![NodeId::new(1), NodeId::GROUND],
            parameters,
        )
    }

    fn pulse() -> PulseSourceModel {
        PulseSourceModel {
            id: DeviceId::new(0),
            mode: PulseMode::Voltage,
            initial: 0.0,
            pulsed: 5.0,
            timing: PulseTiming {
                delay: 1e-6,
                rise: 1e-6,
                fall: 1e-6,
                width: 2e-6,
                period: 10e-6,
            },
            time: 0.0,
            step: None,
            branch: None,
            pins: [NodeId::new(1), NodeId::GROUND],
        }
    }

    #[test]
    fn pulse_follows_rise_high_fall_and_repeat() {
        let model = pulse();
        assert!(model.value_at(0.5e-6).abs() < 1e-12);
        assert!((model.value_at(1.5e-6) - 2.5).abs() < 1e-12);
        assert!(
            (model.value_at(3.0e-6) - 5.0).abs() < 1e-12,
            "v={}",
            model.value_at(3.0e-6)
        );
        assert!((model.value_at(4.5e-6) - 2.5).abs() < 1e-12);
        assert!(model.value_at(5.0e-6).abs() < 1e-12);
        assert!((model.value_at(11.5e-6) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn update_advances_pulse_time() {
        let mut model = pulse();
        model.step = Some(1e-6);
        model.update(&Solution::zero(2, 0)).expect("update");
        assert!((model.time - 1e-6).abs() < 1e-12);
    }

    fn sine() -> SineSourceModel {
        SineSourceModel {
            id: DeviceId::new(0),
            mode: SineMode::Voltage,
            amplitude: 5.0,
            frequency: 1000.0,
            phase: 0.0,
            offset: 0.0,
            time: 0.0,
            step: None,
            branch: None,
            pins: [NodeId::new(1), NodeId::GROUND],
        }
    }

    #[test]
    fn sine_reaches_amplitude_at_quarter_period() {
        let model = sine();
        assert!(model.value_at(0.0).abs() < 1e-12);
        assert!((model.value_at(0.25e-3) - 5.0).abs() < 1e-9);
        assert!((model.value_at(0.75e-3) + 5.0).abs() < 1e-9);
    }

    #[test]
    fn sine_update_advances_time() {
        let mut model = sine();
        model.step = Some(1e-6);
        model.update(&Solution::zero(2, 0)).expect("update");
        assert!((model.time - 1e-6).abs() < 1e-12);
    }

    #[test]
    fn exponential_source_rises_and_falls() {
        let model = ExponentialSourceModel::from_instance(&source_instance(
            DeviceKind::ExponentialVoltageSource,
            &[
                ("v1", 0.0),
                ("v2", 5.0),
                ("td1", 0.0),
                ("tau1", 1e-3),
                ("td2", 1e-3),
                ("tau2", 1e-3),
            ],
        ))
        .expect("model");
        assert!(model.value_at(0.0).abs() < 1e-9);
        assert!((model.value_at(1e-3) - 3.160602794).abs() < 1e-6);
        assert!(model.value_at(2e-3).abs() < 1e-6);
    }

    #[test]
    fn pwl_source_interpolates_points() {
        let model =
            PwlSourceModel::from_instance(&source_instance(DeviceKind::PwlVoltageSource, &[]))
                .expect("model");
        assert!(model.value_at(0.0).abs() < 1e-9);
        assert!((model.value_at(0.5e-3) - 2.5).abs() < 1e-9);
        assert!((model.value_at(2.5e-3) - 2.5).abs() < 1e-9);
        assert!(model.value_at(4e-3).abs() < 1e-9);
    }

    #[test]
    fn polynomial_source_evaluates_coefficients() {
        let model = PolynomialSourceModel::from_instance(&source_instance(
            DeviceKind::PolynomialVoltageSource,
            &[("c0", 1.0), ("c1", 2.0), ("c2", 3.0)],
        ))
        .expect("model");
        assert!((model.value_at(2.0) - 17.0).abs() < 1e-9);
    }

    #[test]
    fn noise_is_deterministic_for_a_seed() {
        let mut first = NoiseSourceModel::from_instance(
            &source_instance(DeviceKind::NoiseVoltageSource, &[("seed", 7.0)]),
            NoiseMode::Voltage,
        )
        .expect("first");
        let mut second = NoiseSourceModel::from_instance(
            &source_instance(DeviceKind::NoiseVoltageSource, &[("seed", 7.0)]),
            NoiseMode::Voltage,
        )
        .expect("second");
        assert!((first.generate() - second.generate()).abs() < 1e-12);
    }
}
