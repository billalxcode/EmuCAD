//! Behavioral three-pin temperature sensor modules.

use crate::model::{
    DeviceModel, non_negative_parameter, parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_current_source,
};
use sim_core::{Device, DeviceInstance, MnaContext, NodeId, SimError, Solution};

const KELVIN: f64 = 273.15;
const REFERENCE_KELVIN: f64 = 298.15;

fn sensor_pins(instance: &DeviceInstance) -> Result<[NodeId; 3], SimError> {
    let pins = pins_exact(instance, 3)?;
    Ok([pins[0], pins[1], pins[2]])
}

fn supply(solution: &Solution, pins: [NodeId; 3]) -> f64 {
    solution.voltage(pins[0]).unwrap_or(0.0) - solution.voltage(pins[1]).unwrap_or(0.0)
}

fn clamp_output(value: f64, supply: f64) -> f64 {
    value.clamp(0.0, supply.max(0.0))
}

fn stamp_output(
    ctx: &mut MnaContext,
    output: NodeId,
    ground: NodeId,
    voltage: f64,
    resistance: f64,
) -> Result<(), SimError> {
    let conductance = 1.0 / resistance.max(1e-9);
    stamp_conductance(ctx, output, ground, conductance)?;
    stamp_current_source(ctx, ground, output, voltage * conductance)
}

fn divider_output(resistance: f64, series: f64, supply: f64) -> (f64, f64) {
    let total = resistance + series;
    let voltage = clamp_output(supply * resistance / total, supply);
    let output_resistance = resistance * series / total;
    (voltage, output_resistance.max(1e-9))
}

fn temperature_value(params: &std::collections::BTreeMap<String, f64>) -> Result<f64, SimError> {
    let temperature = parameter(params, "temperature", 25.0);
    if !temperature.is_finite() || temperature <= -KELVIN {
        return Err(SimError::InvalidNetlist(
            "sensor temperature is out of range".to_string(),
        ));
    }
    Ok(temperature)
}

/// An NTC thermistor module with a series resistor.
pub struct NtcModel {
    resistance: f64,
    beta: f64,
    series: f64,
    temperature: f64,
    pins: [NodeId; 3],
}

impl NtcModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            resistance: positive_parameter(params, "resistance", Some(10000.0))?,
            beta: positive_parameter(params, "beta", Some(3950.0))?,
            series: positive_parameter(params, "series_resistance", Some(10000.0))?,
            temperature: temperature_value(params)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn resistance_at(&self) -> f64 {
        let temperature = self.temperature + KELVIN;
        self.resistance * (self.beta * (1.0 / temperature - 1.0 / REFERENCE_KELVIN)).exp()
    }

    fn output(&self, supply: f64) -> (f64, f64) {
        divider_output(self.resistance_at(), self.series, supply)
    }
}

impl DeviceModel for NtcModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        let (voltage, resistance) = self.output(supply);
        stamp_output(ctx, self.pins[2], self.pins[1], voltage, resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A linear PTC thermistor module with a series resistor.
pub struct PtcModel {
    resistance: f64,
    coefficient: f64,
    series: f64,
    temperature: f64,
    pins: [NodeId; 3],
}

impl PtcModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            resistance: positive_parameter(params, "resistance", Some(1000.0))?,
            coefficient: non_negative_parameter(params, "temperature_coefficient", 0.0039)?,
            series: positive_parameter(params, "series_resistance", Some(1000.0))?,
            temperature: temperature_value(params)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn resistance_at(&self) -> f64 {
        (self.resistance * (1.0 + self.coefficient * (self.temperature - 25.0))).max(1e-9)
    }

    fn output(&self, supply: f64) -> (f64, f64) {
        divider_output(self.resistance_at(), self.series, supply)
    }
}

impl DeviceModel for PtcModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        let (voltage, resistance) = self.output(supply);
        stamp_output(ctx, self.pins[2], self.pins[1], voltage, resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A platinum RTD module using the Callendar-Van Dusen equation.
pub struct RtdModel {
    resistance: f64,
    alpha: f64,
    beta: f64,
    c: f64,
    series: f64,
    temperature: f64,
    pins: [NodeId; 3],
}

impl RtdModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            resistance: positive_parameter(params, "resistance", Some(100.0))?,
            alpha: positive_parameter(params, "alpha", Some(3.9083e-3))?,
            beta: parameter(params, "beta", -5.775e-7),
            c: parameter(params, "c", -4.183e-12),
            series: positive_parameter(params, "series_resistance", Some(1000.0))?,
            temperature: temperature_value(params)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn resistance_at(&self) -> f64 {
        let t = self.temperature;
        let factor = if t < 0.0 {
            1.0 + self.alpha * t + self.beta * t * t + self.c * t * t * t * (t - 100.0)
        } else {
            1.0 + self.alpha * t + self.beta * t * t
        };
        (self.resistance * factor).max(1e-9)
    }

    fn output(&self, supply: f64) -> (f64, f64) {
        divider_output(self.resistance_at(), self.series, supply)
    }
}

impl DeviceModel for RtdModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        let (voltage, resistance) = self.output(supply);
        stamp_output(ctx, self.pins[2], self.pins[1], voltage, resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A thermocouple module with cold-junction compensation.
pub struct ThermocoupleModel {
    sensitivity: f64,
    cold_junction: f64,
    offset: f64,
    output_resistance: f64,
    temperature: f64,
    pins: [NodeId; 3],
}

impl ThermocoupleModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        let cold_junction = parameter(params, "cold_junction", 25.0);
        if !cold_junction.is_finite() {
            return Err(SimError::InvalidNetlist(
                "thermocouple cold junction is invalid".to_string(),
            ));
        }
        Ok(Self {
            sensitivity: positive_parameter(params, "sensitivity", Some(41e-6))?,
            cold_junction,
            offset: parameter(params, "offset", 0.0),
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            temperature: temperature_value(params)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn output(&self, supply: f64) -> f64 {
        clamp_output(
            self.offset + self.sensitivity * (self.temperature - self.cold_junction),
            supply,
        )
    }
}

impl DeviceModel for ThermocoupleModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            self.output(supply),
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A linear semiconductor temperature sensor module.
pub struct TemperatureSensorModel {
    sensitivity: f64,
    offset: f64,
    output_resistance: f64,
    temperature: f64,
    pins: [NodeId; 3],
}

impl TemperatureSensorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            sensitivity: positive_parameter(params, "sensitivity", Some(0.01))?,
            offset: parameter(params, "offset", 0.0),
            output_resistance: positive_parameter(params, "output_resistance", Some(1.0))?,
            temperature: temperature_value(params)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn output(&self, supply: f64) -> f64 {
        clamp_output(self.offset + self.sensitivity * self.temperature, supply)
    }
}

impl DeviceModel for TemperatureSensorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            self.output(supply),
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A temperature switch with hysteresis.
pub struct ThermalSwitchModel {
    threshold: f64,
    hysteresis: f64,
    output_resistance: f64,
    temperature: f64,
    on: bool,
    pins: [NodeId; 3],
}

impl ThermalSwitchModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            threshold: parameter(params, "threshold", 75.0),
            hysteresis: non_negative_parameter(params, "hysteresis", 5.0)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            temperature: temperature_value(params)?,
            on: false,
            pins: sensor_pins(instance)?,
        })
    }

    fn output(&mut self, supply: f64) -> f64 {
        if self.temperature > self.threshold {
            self.on = true;
        }
        if self.temperature < self.threshold - self.hysteresis {
            self.on = false;
        }
        if self.on { supply.max(0.0) } else { 0.0 }
    }
}

impl DeviceModel for ThermalSwitchModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        let output = self.output(supply);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            output,
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A photoresistor module with a gamma resistance model.
pub struct PhotoresistorModel {
    resistance_10lux: f64,
    gamma: f64,
    series: f64,
    illuminance: f64,
    pins: [NodeId; 3],
}

impl PhotoresistorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        Ok(Self {
            resistance_10lux: positive_parameter(params, "resistance_10lux", Some(10000.0))?,
            gamma: positive_parameter(params, "gamma", Some(0.8))?,
            series: positive_parameter(params, "series_resistance", Some(10000.0))?,
            illuminance: non_negative_parameter(params, "illuminance", 10.0)?,
            pins: sensor_pins(instance)?,
        })
    }

    fn resistance_at(&self) -> f64 {
        let lux = self.illuminance.max(1e-9);
        self.resistance_10lux * (lux / 10.0).powf(-self.gamma)
    }

    fn output(&self, supply: f64) -> (f64, f64) {
        divider_output(self.resistance_at(), self.series, supply)
    }
}

impl DeviceModel for PhotoresistorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        let (voltage, resistance) = self.output(supply);
        stamp_output(ctx, self.pins[2], self.pins[1], voltage, resistance)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A photodiode module with a transimpedance output.
pub struct PhotodiodeSensorModel {
    responsivity: f64,
    area: f64,
    feedback: f64,
    irradiance: f64,
    offset: f64,
    output_resistance: f64,
    pins: [NodeId; 3],
}

impl PhotodiodeSensorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        let offset = parameter(params, "offset", 0.0);
        if !offset.is_finite() {
            return Err(SimError::InvalidNetlist(
                "photodiode offset is invalid".to_string(),
            ));
        }
        Ok(Self {
            responsivity: positive_parameter(params, "responsivity", Some(0.5))?,
            area: positive_parameter(params, "area", Some(1e-6))?,
            feedback: positive_parameter(params, "feedback_resistance", Some(1e6))?,
            irradiance: non_negative_parameter(params, "irradiance", 1.0)?,
            offset,
            output_resistance: positive_parameter(params, "output_resistance", Some(100.0))?,
            pins: sensor_pins(instance)?,
        })
    }

    fn output(&self, supply: f64) -> f64 {
        let current = self.responsivity * self.area * self.irradiance;
        clamp_output(self.offset + current * self.feedback, supply)
    }
}

impl DeviceModel for PhotodiodeSensorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            self.output(supply),
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

/// A linear light sensor module in volts per lux.
pub struct LinearLightSensorModel {
    sensitivity: f64,
    offset: f64,
    illuminance: f64,
    output_resistance: f64,
    pins: [NodeId; 3],
}

impl LinearLightSensorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let params = instance.parameters();
        let offset = parameter(params, "offset", 0.0);
        if !offset.is_finite() {
            return Err(SimError::InvalidNetlist(
                "light sensor offset is invalid".to_string(),
            ));
        }
        Ok(Self {
            sensitivity: positive_parameter(params, "sensitivity", Some(0.001))?,
            offset,
            illuminance: non_negative_parameter(params, "illuminance", 1000.0)?,
            output_resistance: positive_parameter(params, "output_resistance", Some(1000.0))?,
            pins: sensor_pins(instance)?,
        })
    }

    fn output(&self, supply: f64) -> f64 {
        clamp_output(self.offset + self.sensitivity * self.illuminance, supply)
    }
}

impl DeviceModel for LinearLightSensorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let supply = supply(solution, self.pins);
        stamp_output(
            ctx,
            self.pins[2],
            self.pins[1],
            self.output(supply),
            self.output_resistance,
        )
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LinearLightSensorModel, NtcModel, PhotodiodeSensorModel, PhotoresistorModel, PtcModel,
        RtdModel, TemperatureSensorModel, ThermalSwitchModel, ThermocoupleModel,
    };
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "s1",
            kind,
            vec![NodeId::new(1), NodeId::GROUND, NodeId::new(2)],
            parameters,
        )
    }

    #[test]
    fn ntc_resistance_falls_with_temperature() {
        let cold = NtcModel::from_instance(&instance(DeviceKind::Ntc, &[("temperature", 25.0)]))
            .expect("cold");
        let hot = NtcModel::from_instance(&instance(DeviceKind::Ntc, &[("temperature", 100.0)]))
            .expect("hot");
        assert!(cold.resistance_at() > hot.resistance_at());
    }

    #[test]
    fn ptc_resistance_rises_with_temperature() {
        let cold = PtcModel::from_instance(&instance(DeviceKind::Ptc, &[("temperature", 25.0)]))
            .expect("cold");
        let hot = PtcModel::from_instance(&instance(DeviceKind::Ptc, &[("temperature", 100.0)]))
            .expect("hot");
        assert!(hot.resistance_at() > cold.resistance_at());
    }

    #[test]
    fn rtd_matches_pt100_at_zero_and_hundred() {
        let zero = RtdModel::from_instance(&instance(DeviceKind::Rtd, &[("temperature", 0.0)]))
            .expect("zero");
        let hundred =
            RtdModel::from_instance(&instance(DeviceKind::Rtd, &[("temperature", 100.0)]))
                .expect("hundred");
        assert!((zero.resistance_at() - 100.0).abs() < 1e-9);
        assert!((hundred.resistance_at() - 138.5).abs() < 0.1);
    }

    #[test]
    fn thermocouple_scales_with_temperature_difference() {
        let model = ThermocoupleModel::from_instance(&instance(
            DeviceKind::Thermocouple,
            &[("temperature", 100.0), ("cold_junction", 25.0)],
        ))
        .expect("model");
        let expected = 41e-6 * 75.0;
        assert!((model.output(5.0) - expected).abs() < 1e-9);
    }

    #[test]
    fn semiconductor_sensor_scales_in_volts_per_degree() {
        let model = TemperatureSensorModel::from_instance(&instance(
            DeviceKind::TemperatureSensor,
            &[("temperature", 25.0)],
        ))
        .expect("model");
        assert!((model.output(5.0) - 0.25).abs() < 1e-9);
    }

    #[test]
    fn thermal_switch_uses_hysteresis() {
        let mut model = ThermalSwitchModel::from_instance(&instance(
            DeviceKind::ThermalSwitch,
            &[("temperature", 80.0)],
        ))
        .expect("model");
        assert!((model.output(5.0) - 5.0).abs() < 1e-9);
        let mut cool = ThermalSwitchModel::from_instance(&instance(
            DeviceKind::ThermalSwitch,
            &[("temperature", 60.0)],
        ))
        .expect("model");
        assert!(cool.output(5.0).abs() < 1e-9);
    }

    #[test]
    fn photoresistor_resistance_falls_with_light() {
        let dark = PhotoresistorModel::from_instance(&instance(
            DeviceKind::Photoresistor,
            &[("illuminance", 10.0)],
        ))
        .expect("dark");
        let bright = PhotoresistorModel::from_instance(&instance(
            DeviceKind::Photoresistor,
            &[("illuminance", 100.0)],
        ))
        .expect("bright");
        assert!((dark.resistance_at() - 10000.0).abs() < 1e-6);
        assert!(bright.resistance_at() < dark.resistance_at());
    }

    #[test]
    fn photodiode_output_matches_transimpedance() {
        let model =
            PhotodiodeSensorModel::from_instance(&instance(DeviceKind::PhotodiodeSensor, &[]))
                .expect("model");
        assert!((model.output(5.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn linear_light_output_matches_sensitivity() {
        let model = LinearLightSensorModel::from_instance(&instance(
            DeviceKind::LinearLightSensor,
            &[("sensitivity", 0.001), ("illuminance", 1000.0)],
        ))
        .expect("model");
        assert!((model.output(5.0) - 1.0).abs() < 1e-9);
    }
}
