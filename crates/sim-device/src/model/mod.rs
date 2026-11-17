//! Device model API and factory.

use crate::actuator::{DisplayModuleModel, LedActuatorModel, SeriesEmfModel};
use crate::bjt::BjtModel;
use crate::comparator::ComparatorModel;
use crate::controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
use crate::converter::ConverterModel;
use crate::crystal::CrystalModel;
use crate::detector::{BehavioralDetectorModel, DetectorKind};
use crate::diode::DiodeModel;
use crate::diode_array::{DiodeBridgeModel, DualDiodeModel};
use crate::driver::MotorDriverModel;
use crate::filter::{BehavioralFilterModel, FilterKind};
use crate::igbt::IgbtModel;
use crate::jfet::JfetModel;
use crate::logic::LogicGateModel;
use crate::math::{BehavioralMathModel, MathKind};
use crate::mosfet::MosfetModel;
use crate::opamp::OpAmpModel;
use crate::optocoupler::OptocouplerModel;
use crate::oscillator::{BehavioralOscillatorModel, OscillatorKind};
use crate::passive::{CapacitorModel, InductorModel, ResistorModel};
use crate::protection::{FuseModel, VaristorModel};
use crate::regulator::RegulatorModel;
use crate::relay::RelayModel;
use crate::rf::RfAmplifierModel;
use crate::sensors::{
    CurrentSenseModel, LinearLightSensorModel, LinearSensorModel, LoadCellModel,
    MagneticFieldSensorModel, NtcModel, PhSensorModel, PhotodiodeSensorModel, PhotoresistorModel,
    PtcModel, RelativeResistanceSensorModel, RtdModel, StrainGaugeSensorModel,
    TemperatureSensorModel, ThermalSwitchModel, ThermocoupleModel, ThresholdSwitchModel,
};
use crate::source::{
    CurrentSourceModel, ExponentialSourceModel, NoiseSourceModel, PulseSourceModel, PwlSourceModel,
    SineSourceModel, VoltageSourceModel,
};
use crate::supervisor::{CurrentReferenceModel, SupervisorModel, VoltageReferenceModel};
use crate::switch::{IdealSwitchModel, SwitchModel};
use crate::thyristor::ThyristorModel;
use crate::timer::{AstableTimerModel, MonostableTimerModel, VcoTimerModel};
use crate::transformer::TransformerModel;
use crate::transmission::TransmissionLineModel;
use crate::unijunction::UnijunctionModel;
use sim_core::{
    BOLTZMANN, Device, DeviceId, DeviceInstance, DeviceKind, ELEMENTARY_CHARGE, MnaContext,
    NOMINAL_TEMPERATURE, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// Maximum exponent used by junction models.
pub(crate) const MAX_EXPONENT: f64 = 40.0;

/// Maximum forward junction voltage used for limiting.
pub(crate) const MAX_FORWARD_VOLTAGE: f64 = 0.9;

/// Numerical integration method for reactive devices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntegrationMethod {
    /// Backward Euler.
    #[default]
    BackwardEuler,
    /// Trapezoidal.
    Trapezoidal,
    /// Second-order Gear.
    Gear2,
}

/// A device model that can stamp an MNA system.
pub trait DeviceModel {
    /// Stamps the device into the MNA system.
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError>;

    /// Stamps with an explicit integration method.
    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
        _method: IntegrationMethod,
    ) -> Result<(), SimError> {
        self.stamp(ctx, solution, dt)
    }

    /// Updates internal state after a converged solution.
    fn update(&mut self, solution: &Solution) -> Result<(), SimError>;

    /// Returns the device current for probes when available.
    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        None
    }
}

/// Builds a model from a structural device instance.
pub fn model_from_instance(instance: &DeviceInstance) -> Result<Box<dyn DeviceModel>, SimError> {
    if let Some(model) = passive_model(instance)? {
        return Ok(model);
    }
    if let Some(model) = active_model(instance)? {
        return Ok(model);
    }
    Err(SimError::UnsupportedDevice(
        instance.kind().name().to_string(),
    ))
}

fn passive_model(instance: &DeviceInstance) -> Result<Option<Box<dyn DeviceModel>>, SimError> {
    let model: Box<dyn DeviceModel> = match instance.kind() {
        DeviceKind::Resistor => Box::new(ResistorModel::from_instance(instance)?),
        DeviceKind::Capacitor => Box::new(CapacitorModel::from_instance(instance)?),
        DeviceKind::Inductor => Box::new(InductorModel::from_instance(instance)?),
        DeviceKind::VoltageSource => Box::new(VoltageSourceModel::from_instance(instance)?),
        DeviceKind::PulseVoltageSource => Box::new(PulseSourceModel::voltage(instance)?),
        DeviceKind::PulseCurrentSource => Box::new(PulseSourceModel::current(instance)?),
        DeviceKind::ExponentialVoltageSource => {
            Box::new(ExponentialSourceModel::from_instance(instance)?)
        }
        DeviceKind::PwlVoltageSource => Box::new(PwlSourceModel::from_instance(instance)?),
        DeviceKind::NoiseVoltageSource => Box::new(NoiseSourceModel::voltage(instance)?),
        DeviceKind::NoiseCurrentSource => Box::new(NoiseSourceModel::current(instance)?),
        DeviceKind::BehavioralIntegrator => Box::new(BehavioralFilterModel::from_instance(
            instance,
            FilterKind::Integrator,
        )?),
        DeviceKind::BehavioralDifferentiator => Box::new(BehavioralFilterModel::from_instance(
            instance,
            FilterKind::Differentiator,
        )?),
        DeviceKind::BehavioralLowPass => Box::new(BehavioralFilterModel::from_instance(
            instance,
            FilterKind::LowPass,
        )?),
        DeviceKind::BehavioralHighPass => Box::new(BehavioralFilterModel::from_instance(
            instance,
            FilterKind::HighPass,
        )?),
        DeviceKind::BehavioralStateVariable => Box::new(BehavioralFilterModel::from_instance(
            instance,
            FilterKind::StateVariable,
        )?),
        DeviceKind::BehavioralMultiplier => Box::new(BehavioralMathModel::from_instance(
            instance,
            MathKind::Multiplier,
        )?),
        DeviceKind::BehavioralDivider => Box::new(BehavioralMathModel::from_instance(
            instance,
            MathKind::Divider,
        )?),
        DeviceKind::BehavioralLog => Box::new(BehavioralMathModel::from_instance(
            instance,
            MathKind::Logarithmic,
        )?),
        DeviceKind::BehavioralAntilog => Box::new(BehavioralMathModel::from_instance(
            instance,
            MathKind::Antilog,
        )?),
        DeviceKind::BehavioralRms => {
            Box::new(BehavioralMathModel::from_instance(instance, MathKind::Rms)?)
        }
        DeviceKind::BehavioralComparator => Box::new(BehavioralDetectorModel::from_instance(
            instance,
            DetectorKind::Comparator,
        )?),
        DeviceKind::BehavioralSchmitt => Box::new(BehavioralDetectorModel::from_instance(
            instance,
            DetectorKind::Schmitt,
        )?),
        DeviceKind::BehavioralPeakDetector => Box::new(BehavioralDetectorModel::from_instance(
            instance,
            DetectorKind::Peak,
        )?),
        DeviceKind::BehavioralTrackHold => Box::new(BehavioralDetectorModel::from_instance(
            instance,
            DetectorKind::TrackHold,
        )?),
        DeviceKind::BehavioralPrecisionRectifier => Box::new(
            BehavioralDetectorModel::from_instance(instance, DetectorKind::Rectifier)?,
        ),
        DeviceKind::BehavioralVco => Box::new(BehavioralOscillatorModel::from_instance(
            instance,
            OscillatorKind::Vco,
        )?),
        DeviceKind::BehavioralAmplitude => Box::new(BehavioralOscillatorModel::from_instance(
            instance,
            OscillatorKind::Amplitude,
        )?),
        DeviceKind::BehavioralFrequency => Box::new(BehavioralOscillatorModel::from_instance(
            instance,
            OscillatorKind::Frequency,
        )?),
        DeviceKind::BehavioralPwm => Box::new(BehavioralOscillatorModel::from_instance(
            instance,
            OscillatorKind::Pwm,
        )?),
        DeviceKind::CurrentSource => Box::new(CurrentSourceModel::from_instance(instance)?),
        DeviceKind::Vcvs => Box::new(VcvsModel::from_instance(instance)?),
        DeviceKind::Vccs => Box::new(VccsModel::from_instance(instance)?),
        DeviceKind::Cccs => Box::new(CccsModel::from_instance(instance)?),
        DeviceKind::Ccvs => Box::new(CcvsModel::from_instance(instance)?),
        DeviceKind::Switch => Box::new(SwitchModel::from_instance(instance)?),
        DeviceKind::IdealSwitch => Box::new(IdealSwitchModel::from_instance(instance)?),
        DeviceKind::Relay => Box::new(RelayModel::from_instance(instance)?),
        DeviceKind::TransmissionLine => Box::new(TransmissionLineModel::from_instance(instance)?),
        DeviceKind::Crystal => Box::new(CrystalModel::from_instance(instance)?),
        DeviceKind::Transformer => Box::new(TransformerModel::from_instance(instance)?),
        DeviceKind::Varistor => Box::new(VaristorModel::from_instance(instance)?),
        DeviceKind::Fuse => Box::new(FuseModel::from_instance(instance)?),
        _ => return Ok(None),
    };
    Ok(Some(model))
}

fn active_model(instance: &DeviceInstance) -> Result<Option<Box<dyn DeviceModel>>, SimError> {
    let model: Box<dyn DeviceModel> = match instance.kind() {
        DeviceKind::Nmos | DeviceKind::Pmos => Box::new(MosfetModel::from_instance(instance)?),
        DeviceKind::Npn | DeviceKind::Pnp => Box::new(BjtModel::from_instance(instance)?),
        DeviceKind::JfetN | DeviceKind::JfetP => Box::new(JfetModel::from_instance(instance)?),
        DeviceKind::Igbt => Box::new(IgbtModel::from_instance(instance)?),
        DeviceKind::Ujt | DeviceKind::Put => Box::new(UnijunctionModel::from_instance(instance)?),
        DeviceKind::Scr
        | DeviceKind::Triac
        | DeviceKind::Diac
        | DeviceKind::Gto
        | DeviceKind::Scs => Box::new(ThyristorModel::from_instance(instance)?),
        DeviceKind::OpAmp => Box::new(OpAmpModel::from_instance(instance)?),
        DeviceKind::Comparator => Box::new(ComparatorModel::from_instance(instance)?),
        DeviceKind::Regulator => Box::new(RegulatorModel::from_instance(instance)?),
        DeviceKind::Optocoupler => Box::new(OptocouplerModel::from_instance(instance)?),
        DeviceKind::Ntc => Box::new(NtcModel::from_instance(instance)?),
        DeviceKind::Ptc => Box::new(PtcModel::from_instance(instance)?),
        DeviceKind::Rtd => Box::new(RtdModel::from_instance(instance)?),
        DeviceKind::Thermocouple => Box::new(ThermocoupleModel::from_instance(instance)?),
        DeviceKind::TemperatureSensor => Box::new(TemperatureSensorModel::from_instance(instance)?),
        DeviceKind::ThermalSwitch => Box::new(ThermalSwitchModel::from_instance(instance)?),
        DeviceKind::Photoresistor => Box::new(PhotoresistorModel::from_instance(instance)?),
        DeviceKind::PhotodiodeSensor => Box::new(PhotodiodeSensorModel::from_instance(instance)?),
        DeviceKind::LinearLightSensor => Box::new(LinearLightSensorModel::from_instance(instance)?),
        DeviceKind::HallSwitch | DeviceKind::ReedSwitchSensor => Box::new(
            ThresholdSwitchModel::from_instance(instance, "magnetic_field", false, 0.005, 0.001)?,
        ),
        DeviceKind::LinearHallSensor => {
            Box::new(MagneticFieldSensorModel::from_instance(instance, 5.0, 2.5)?)
        }
        DeviceKind::FluxgateSensor => Box::new(MagneticFieldSensorModel::from_instance(
            instance, 1000.0, 0.0,
        )?),
        DeviceKind::MagnetoresistiveSensor => {
            Box::new(RelativeResistanceSensorModel::from_instance(
                instance,
                "magnetic_field",
                1000.0,
                10.0,
                1000.0,
            )?)
        }
        DeviceKind::Accelerometer | DeviceKind::Imu => Box::new(LinearSensorModel::from_instance(
            instance,
            "acceleration",
            0.050986,
            2.5,
        )?),
        DeviceKind::Gyroscope => Box::new(LinearSensorModel::from_instance(
            instance,
            "angular_rate",
            0.01,
            2.5,
        )?),
        DeviceKind::TiltSensor => Box::new(LinearSensorModel::from_instance(
            instance, "angle", 0.025, 2.5,
        )?),
        DeviceKind::VibrationSensor => Box::new(LinearSensorModel::from_instance(
            instance,
            "vibration",
            0.050986,
            2.5,
        )?),
        DeviceKind::StrainGaugeSensor => Box::new(StrainGaugeSensorModel::from_instance(instance)?),
        DeviceKind::LoadCell => Box::new(LoadCellModel::from_instance(instance)?),
        DeviceKind::PressureSensor => Box::new(LinearSensorModel::from_instance(
            instance, "pressure", 1e-5, 0.5,
        )?),
        DeviceKind::ForceSensor => Box::new(LinearSensorModel::from_instance(
            instance, "force", 0.01, 0.0,
        )?),
        DeviceKind::HumiditySensor => Box::new(LinearSensorModel::from_instance(
            instance,
            "relative_humidity",
            0.03,
            0.8,
        )?),
        DeviceKind::GasSensor => Box::new(RelativeResistanceSensorModel::from_instance(
            instance,
            "gas_concentration",
            10000.0,
            -0.0001,
            10000.0,
        )?),
        DeviceKind::PhSensor => Box::new(PhSensorModel::from_instance(instance)?),
        DeviceKind::ElectrochemicalSensor => Box::new(LinearSensorModel::from_instance(
            instance,
            "concentration",
            0.01,
            0.0,
        )?),
        DeviceKind::CurrentSenseResistor => Box::new(CurrentSenseModel::from_instance(instance)?),
        DeviceKind::HallCurrentSensor => Box::new(LinearSensorModel::from_instance(
            instance, "current", 0.1, 2.5,
        )?),
        DeviceKind::VoltageDividerSensor => Box::new(LinearSensorModel::from_instance(
            instance, "voltage", 0.2, 0.0,
        )?),
        DeviceKind::InductiveProximitySensor => Box::new(ThresholdSwitchModel::from_instance(
            instance, "distance", true, 5.0, 0.5,
        )?),
        DeviceKind::CapacitiveTouchSensor => Box::new(ThresholdSwitchModel::from_instance(
            instance,
            "capacitance",
            false,
            10.0,
            2.0,
        )?),
        DeviceKind::UltrasonicDistanceSensor => Box::new(LinearSensorModel::from_instance(
            instance, "distance", 1.0, 0.5,
        )?),
        DeviceKind::LedActuator => Box::new(LedActuatorModel::from_instance(instance)?),
        DeviceKind::DcMotor => Box::new(SeriesEmfModel::from_instance(
            instance,
            "speed",
            "back_emf_constant",
            10.0,
            0.01,
        )?),
        DeviceKind::Peltier => Box::new(SeriesEmfModel::from_instance(
            instance,
            "delta_temperature",
            "seebeck",
            2.0,
            0.05,
        )?),
        DeviceKind::ThermoelectricGenerator => Box::new(SeriesEmfModel::from_instance(
            instance,
            "delta_temperature",
            "seebeck",
            5.0,
            0.05,
        )?),
        DeviceKind::DisplayModule => Box::new(DisplayModuleModel::from_instance(instance)?),
        DeviceKind::RfAmplifier => Box::new(RfAmplifierModel::from_instance(instance)?),
        DeviceKind::DcDcConverter => Box::new(ConverterModel::from_four_pin(instance, 12.0, 0.85)?),
        DeviceKind::ChargePump => Box::new(ConverterModel::from_three_pin(instance, 5.0, 0.8)?),
        DeviceKind::VoltageReference => Box::new(VoltageReferenceModel::from_instance(instance)?),
        DeviceKind::CurrentReference => Box::new(CurrentReferenceModel::from_instance(instance)?),
        DeviceKind::Supervisor => Box::new(SupervisorModel::from_instance(instance)?),
        DeviceKind::MotorDriver => Box::new(MotorDriverModel::from_instance(instance)?),
        DeviceKind::SineVoltageSource => Box::new(SineSourceModel::voltage(instance)?),
        DeviceKind::SineCurrentSource => Box::new(SineSourceModel::current(instance)?),
        DeviceKind::Timer555 | DeviceKind::AstableTimer => {
            Box::new(AstableTimerModel::from_instance(instance)?)
        }
        DeviceKind::MonostableTimer => Box::new(MonostableTimerModel::from_instance(instance)?),
        DeviceKind::VcoTimer => Box::new(VcoTimerModel::from_instance(instance)?),
        DeviceKind::LogicGate => Box::new(LogicGateModel::from_instance(instance)?),
        DeviceKind::Diode => Box::new(DiodeModel::from_instance(instance)?),
        DeviceKind::DiodeBridge => Box::new(DiodeBridgeModel::from_instance(instance)?),
        DeviceKind::DualDiode => Box::new(DualDiodeModel::from_instance(instance)?),
        _ => return Ok(None),
    };
    Ok(Some(model))
}

mod parameters;
mod stamps;
#[cfg(test)]
mod tests;

pub(crate) use parameters::*;
pub(crate) use stamps::*;
