//! Device models for the simulation core.

#![forbid(unsafe_code)]

mod actuator;
mod behavioral_converter;
mod bjt;
mod catalog;
mod comparator;
mod controlled;
mod converter;
mod crystal;
mod detector;
mod diode;
mod diode_array;
mod driver;
mod filter;
mod igbt;
mod jfet;
mod logic;
mod math;
mod model;
mod mosfet;
mod opamp;
mod optocoupler;
mod oscillator;
mod passive;
mod protection;
mod regulator;
mod relay;
mod rf;
mod sensors;
mod source;
mod supervisor;
mod switch;
mod thyristor;
mod timer;
mod transformer;
mod transmission;
mod unijunction;

pub use actuator::{DisplayModuleModel, LedActuatorModel, SeriesEmfModel};
pub use behavioral_converter::BehavioralConverterModel;
pub use bjt::{BjtModel, bjt_capacitances};
pub use catalog::{
    CatalogError, CatalogRecord, ComponentRegistry, ComponentRequest, ComponentSpec, Family,
    Fidelity, ModelClass, ParameterKind, ParameterSpec, PinRole, SimulationBinding, catalog,
    catalog_records, device_parameters, device_pin_roles, find_catalog_record, simulation_binding,
    simulation_bindings,
};
pub use comparator::ComparatorModel;
pub use controlled::{CccsModel, CcvsModel, VccsModel, VcvsModel};
pub use converter::ConverterModel;
pub use crystal::CrystalModel;
pub use detector::BehavioralDetectorModel;
pub use diode::{DiodeModel, diode_capacitance};
pub use diode_array::{DiodeBridgeModel, DualDiodeModel};
pub use driver::MotorDriverModel;
pub use filter::BehavioralFilterModel;
pub use igbt::IgbtModel;
pub use jfet::{JfetModel, jfet_capacitances};
pub use logic::LogicGateModel;
pub use math::BehavioralMathModel;
pub use model::{DeviceModel, IntegrationMethod, model_from_instance};
pub use mosfet::{MosfetModel, MosfetParams, mosfet_capacitances};
pub use opamp::OpAmpModel;
pub use optocoupler::OptocouplerModel;
pub use oscillator::BehavioralOscillatorModel;
pub use passive::{CapacitorModel, InductorModel, ResistorModel};
pub use protection::{FuseModel, VaristorModel};
pub use regulator::RegulatorModel;
pub use relay::RelayModel;
pub use rf::RfAmplifierModel;
pub use sensors::{
    CurrentSenseModel, LinearLightSensorModel, LinearSensorModel, LoadCellModel,
    MagneticFieldSensorModel, NtcModel, PhSensorModel, PhotodiodeSensorModel, PhotoresistorModel,
    PtcModel, RelativeResistanceSensorModel, RtdModel, StrainGaugeSensorModel,
    TemperatureSensorModel, ThermalSwitchModel, ThermocoupleModel, ThresholdSwitchModel,
};
pub use source::{
    CurrentSourceModel, ExponentialSourceModel, NoiseSourceModel, PolynomialSourceModel,
    PulseSourceModel, PwlSourceModel, SineSourceModel, VoltageSourceModel,
};
pub use supervisor::{CurrentReferenceModel, SupervisorModel, VoltageReferenceModel};
pub use switch::{IdealSwitchModel, SwitchModel};
pub use thyristor::{ThyristorKind, ThyristorModel};
pub use timer::{AstableTimerModel, MonostableTimerModel, VcoTimerModel};
pub use transformer::TransformerModel;
pub use transmission::TransmissionLineModel;
pub use unijunction::{UnijunctionMode, UnijunctionModel};
