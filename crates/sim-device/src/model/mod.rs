//! Device model API and factory.

use crate::bjt::BjtModel;
use crate::comparator::ComparatorModel;
use crate::controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
use crate::crystal::CrystalModel;
use crate::diode::DiodeModel;
use crate::diode_array::{DiodeBridgeModel, DualDiodeModel};
use crate::igbt::IgbtModel;
use crate::jfet::JfetModel;
use crate::logic::LogicGateModel;
use crate::mosfet::MosfetModel;
use crate::opamp::OpAmpModel;
use crate::optocoupler::OptocouplerModel;
use crate::passive::{CapacitorModel, InductorModel, ResistorModel};
use crate::protection::{FuseModel, VaristorModel};
use crate::regulator::RegulatorModel;
use crate::relay::RelayModel;
use crate::sensors::{
    LinearLightSensorModel, NtcModel, PhotodiodeSensorModel, PhotoresistorModel, PtcModel,
    RtdModel, TemperatureSensorModel, ThermalSwitchModel, ThermocoupleModel,
};
use crate::source::{CurrentSourceModel, PulseSourceModel, VoltageSourceModel};
use crate::switch::{IdealSwitchModel, SwitchModel};
use crate::thyristor::ThyristorModel;
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
        DeviceKind::PulseVoltageSource => Box::new(PulseSourceModel::from_instance(instance)?),
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
