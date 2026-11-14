//! Catalog types, families, and model-class behavior.

use sim_core::DeviceKind;

/// Top-level component family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    Passive,
    Source,
    Diode,
    Transistor,
    Protection,
    Switch,
    Analog,
    Logic,
    Memory,
    Mcu,
    Power,
    Sensor,
    Actuator,
    Rf,
    Behavioral,
    Board,
}

/// Generic model class used by the catalog and the Python builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelClass {
    Resistor,
    Capacitor,
    Inductor,
    Transformer,
    Crystal,
    TransmissionLine,
    VoltageSource,
    CurrentSource,
    Vcvs,
    Vccs,
    Cccs,
    Ccvs,
    BehavioralSource,
    DigitalSource,
    Battery,
    Diode,
    DiodeArray,
    Led,
    Photodiode,
    Optocoupler,
    Bjt,
    Jfet,
    Mosfet,
    OtherTransistor,
    Thyristor,
    Protection,
    Switch,
    Relay,
    SolidStateRelay,
    AnalogSwitch,
    Connector,
    OpAmp,
    Comparator,
    Timer,
    VoltageReference,
    LogicGate,
    LogicGateSpecial,
    FlipFlop,
    Counter,
    Multiplexer,
    Adder,
    LevelShifter,
    ProgrammableLogic,
    Memory,
    Mcu,
    Adc,
    Dac,
    SerialInterface,
    TimerPeripheral,
    Regulator,
    DcDc,
    ChargePump,
    Supervisor,
    PowerSwitch,
    MotorDriver,
    Sensor,
    Actuator,
    Display,
    RfBlock,
    BehavioralBlock,
    BoardModule,
}

/// One catalog entry.
#[derive(Clone, Debug, PartialEq)]
pub struct ComponentSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub class: ModelClass,
    pub device: Option<DeviceKind>,
    pub pins: u8,
    pub parameter: &'static str,
    pub default: f64,
    pub unit: &'static str,
    pub variant: u8,
}

/// Catalog validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogError {
    pub id: &'static str,
    pub reason: &'static str,
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.id, self.reason)
    }
}

impl std::error::Error for CatalogError {}

/// Returns all 500 generic component entries.
mod class;
mod component;
mod family;
mod response;
