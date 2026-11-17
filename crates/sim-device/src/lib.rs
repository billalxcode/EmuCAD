//! Device models for the simulation core.

#![forbid(unsafe_code)]

mod bjt;
mod catalog;
mod comparator;
mod controlled;
mod crystal;
mod diode;
mod diode_array;
mod igbt;
mod jfet;
mod logic;
mod model;
mod mosfet;
mod opamp;
mod optocoupler;
mod passive;
mod protection;
mod regulator;
mod relay;
mod sensors;
mod source;
mod switch;
mod thyristor;
mod transformer;
mod transmission;
mod unijunction;

pub use bjt::{BjtModel, bjt_capacitances};
pub use catalog::{
    CatalogError, CatalogRecord, ComponentRegistry, ComponentRequest, ComponentSpec, Family,
    Fidelity, ModelClass, ParameterKind, ParameterSpec, PinRole, SimulationBinding, catalog,
    catalog_records, device_parameters, device_pin_roles, find_catalog_record, simulation_binding,
    simulation_bindings,
};
pub use comparator::ComparatorModel;
pub use controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
pub use crystal::CrystalModel;
pub use diode::{DiodeModel, diode_capacitance};
pub use diode_array::{DiodeBridgeModel, DualDiodeModel};
pub use igbt::IgbtModel;
pub use jfet::{JfetModel, jfet_capacitances};
pub use logic::LogicGateModel;
pub use model::{DeviceModel, IntegrationMethod, model_from_instance};
pub use mosfet::{MosfetModel, MosfetParams, mosfet_capacitances};
pub use opamp::OpAmpModel;
pub use optocoupler::OptocouplerModel;
pub use passive::{CapacitorModel, InductorModel, ResistorModel};
pub use protection::{FuseModel, VaristorModel};
pub use regulator::RegulatorModel;
pub use relay::RelayModel;
pub use sensors::{
    LinearLightSensorModel, NtcModel, PhotodiodeSensorModel, PhotoresistorModel, PtcModel,
    RtdModel, TemperatureSensorModel, ThermalSwitchModel, ThermocoupleModel,
};
pub use source::{CurrentSourceModel, PulseSourceModel, VoltageSourceModel};
pub use switch::{IdealSwitchModel, SwitchModel};
pub use thyristor::{ThyristorKind, ThyristorModel};
pub use transformer::TransformerModel;
pub use transmission::TransmissionLineModel;
pub use unijunction::{UnijunctionMode, UnijunctionModel};
