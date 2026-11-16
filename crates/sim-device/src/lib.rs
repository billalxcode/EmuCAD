//! Device models for the simulation core.

#![forbid(unsafe_code)]

mod bjt;
mod catalog;
mod comparator;
mod controlled;
mod crystal;
mod diode;
mod igbt;
mod jfet;
mod logic;
mod model;
mod mosfet;
mod opamp;
mod passive;
mod protection;
mod regulator;
mod relay;
mod source;
mod switch;
mod thyristor;
mod transformer;
mod transmission;
mod unijunction;

pub use bjt::{BjtModel, bjt_capacitances};
pub use catalog::{
    CatalogError, CatalogRecord, ComponentRegistry, ComponentRequest, ComponentSpec, Family,
    ModelClass, ParameterKind, ParameterSpec, PinRole, catalog, catalog_records, device_parameters,
    device_pin_roles, find_catalog_record,
};
pub use comparator::ComparatorModel;
pub use controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
pub use crystal::CrystalModel;
pub use diode::{DiodeModel, diode_capacitance};
pub use igbt::IgbtModel;
pub use jfet::{JfetModel, jfet_capacitances};
pub use logic::LogicGateModel;
pub use model::{DeviceModel, IntegrationMethod, model_from_instance};
pub use mosfet::{MosfetModel, MosfetParams, mosfet_capacitances};
pub use opamp::OpAmpModel;
pub use passive::{CapacitorModel, InductorModel, ResistorModel};
pub use protection::{FuseModel, VaristorModel};
pub use regulator::RegulatorModel;
pub use relay::RelayModel;
pub use source::{CurrentSourceModel, PulseSourceModel, VoltageSourceModel};
pub use switch::{IdealSwitchModel, SwitchModel};
pub use thyristor::{ThyristorKind, ThyristorModel};
pub use transformer::TransformerModel;
pub use transmission::TransmissionLineModel;
pub use unijunction::{UnijunctionMode, UnijunctionModel};
