//! Structural device model and solver-facing types.

use crate::ids::{DeviceId, NodeId};
use crate::matrix::DenseMatrix;
use std::collections::BTreeMap;

/// The kind of a device in a structural netlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// An ideal resistor.
    Resistor,
    /// An ideal capacitor.
    Capacitor,
    /// An ideal inductor.
    Inductor,
    /// An independent voltage source.
    VoltageSource,
    /// An independent pulse voltage source.
    PulseVoltageSource,
    /// An independent current source.
    CurrentSource,
    /// A voltage-controlled voltage source.
    Vcvs,
    /// A voltage-controlled current source.
    Vccs,
    /// A current-controlled current source.
    Cccs,
    /// A current-controlled voltage source.
    Ccvs,
    /// A voltage-controlled switch.
    Switch,
    /// A two-terminal ideal switch.
    IdealSwitch,
    /// An electromechanical relay with coil and changeover contacts.
    Relay,
    /// A five-terminal operational amplifier.
    OpAmp,
    /// A five-terminal comparator with saturated output.
    Comparator,
    /// A three-terminal linear regulator.
    Regulator,
    /// A five-terminal logic gate.
    LogicGate,
    /// An N-channel MOSFET.
    Nmos,
    /// A P-channel MOSFET.
    Pmos,
    /// An NPN bipolar junction transistor.
    Npn,
    /// A PNP bipolar junction transistor.
    Pnp,
    /// An N-channel JFET.
    JfetN,
    /// A P-channel JFET.
    JfetP,
    /// A lossless transmission line.
    TransmissionLine,
    /// A piezoelectric crystal with a BVD model.
    Crystal,
    /// A two-winding transformer with mutual inductance.
    Transformer,
    /// A voltage-clamping protection device.
    Varistor,
    /// A current-triggered fuse or breaker.
    Fuse,
    /// An insulated-gate bipolar transistor.
    Igbt,
    /// A unijunction transistor.
    Ujt,
    /// A programmable unijunction transistor.
    Put,
    /// A silicon controlled rectifier.
    Scr,
    /// A bidirectional triode thyristor.
    Triac,
    /// A bidirectional trigger diode.
    Diac,
    /// A gate turn-off thyristor.
    Gto,
    /// A silicon controlled switch.
    Scs,
    /// A four-diode bridge with two AC and two DC pins.
    DiodeBridge,
    /// Two independent diodes in one package.
    DualDiode,
    /// An optically coupled LED and output transistor.
    Optocoupler,
    /// An NTC thermistor sensor module.
    Ntc,
    /// A PTC thermistor sensor module.
    Ptc,
    /// A platinum resistance temperature detector module.
    Rtd,
    /// A thermocouple module.
    Thermocouple,
    /// A linear semiconductor temperature sensor module.
    TemperatureSensor,
    /// A temperature switch with hysteresis.
    ThermalSwitch,
    /// A photoresistor module.
    Photoresistor,
    /// A photodiode transimpedance module.
    PhotodiodeSensor,
    /// A linear light sensor module.
    LinearLightSensor,
    /// A Hall-effect switch with hysteresis.
    HallSwitch,
    /// A linear Hall-effect sensor module.
    LinearHallSensor,
    /// A reed switch sensor module.
    ReedSwitchSensor,
    /// A magnetoresistive sensor module.
    MagnetoresistiveSensor,
    /// A fluxgate magnetic field sensor module.
    FluxgateSensor,
    /// A single-axis accelerometer module.
    Accelerometer,
    /// A single-axis gyroscope module.
    Gyroscope,
    /// A single-axis inertial measurement unit module.
    Imu,
    /// A tilt sensor module.
    TiltSensor,
    /// A vibration sensor module.
    VibrationSensor,
    /// A strain gauge sensor module.
    StrainGaugeSensor,
    /// A load cell module.
    LoadCell,
    /// A pressure sensor module.
    PressureSensor,
    /// A force sensor module.
    ForceSensor,
    /// A humidity sensor module.
    HumiditySensor,
    /// A gas sensor module.
    GasSensor,
    /// A pH sensor module.
    PhSensor,
    /// An electrochemical sensor module.
    ElectrochemicalSensor,
    /// A current sense resistor module.
    CurrentSenseResistor,
    /// A Hall-effect current sensor module.
    HallCurrentSensor,
    /// A voltage divider sensor module.
    VoltageDividerSensor,
    /// An inductive proximity sensor module.
    InductiveProximitySensor,
    /// A capacitive touch sensor module.
    CapacitiveTouchSensor,
    /// An ultrasonic distance sensor module.
    UltrasonicDistanceSensor,
    /// An LED actuator module.
    LedActuator,
    /// A DC motor module with back EMF.
    DcMotor,
    /// A Peltier cooler module.
    Peltier,
    /// A thermoelectric generator module.
    ThermoelectricGenerator,
    /// A four-pin display module.
    DisplayModule,
    /// A three-pin RF amplifier module.
    RfAmplifier,
    /// A four-pin DC-DC converter module.
    DcDcConverter,
    /// A three-pin charge pump or charger module.
    ChargePump,
    /// A three-pin voltage reference module.
    VoltageReference,
    /// A three-pin current reference module.
    CurrentReference,
    /// A three-pin supply supervisor module.
    Supervisor,
    /// A four-pin motor driver module.
    MotorDriver,
    /// A sine voltage source.
    SineVoltageSource,
    /// A sine current source.
    SineCurrentSource,
    /// A 555 timer module.
    Timer555,
    /// A monostable timer module.
    MonostableTimer,
    /// An astable timer module.
    AstableTimer,
    /// A voltage-controlled oscillator module.
    VcoTimer,
    /// A pulse current source.
    PulseCurrentSource,
    /// An exponential voltage source.
    ExponentialVoltageSource,
    /// A piecewise-linear voltage source.
    PwlVoltageSource,
    /// A deterministic voltage noise source.
    NoiseVoltageSource,
    /// A deterministic current noise source.
    NoiseCurrentSource,
    /// A behavioral integrator block.
    BehavioralIntegrator,
    /// A behavioral differentiator block.
    BehavioralDifferentiator,
    /// A behavioral low-pass filter block.
    BehavioralLowPass,
    /// A behavioral high-pass filter block.
    BehavioralHighPass,
    /// A behavioral state-variable filter block.
    BehavioralStateVariable,
    /// A diode.
    Diode,
    /// A model supplied by another crate or a future device library.
    Custom(String),
}

impl DeviceKind {
    /// Returns a stable name for the device kind.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Custom(name) => name,
            other => other.static_name(),
        }
    }

    fn static_name(&self) -> &'static str {
        match self {
            Self::Resistor => "resistor",
            Self::Capacitor => "capacitor",
            Self::Inductor => "inductor",
            Self::VoltageSource | Self::PulseVoltageSource | Self::CurrentSource => {
                self.source_name()
            }
            Self::Vcvs | Self::Vccs | Self::Cccs | Self::Ccvs => self.controlled_name(),
            Self::Switch => "switch",
            Self::IdealSwitch => "ideal_switch",
            Self::Relay => "relay",
            Self::OpAmp => "opamp",
            Self::Comparator => "comparator",
            Self::Regulator => "regulator",
            Self::LogicGate => "logic_gate",
            other => other.analog_name(),
        }
    }

    fn source_name(&self) -> &'static str {
        match self {
            Self::PulseVoltageSource => "pulse_voltage_source",
            Self::CurrentSource => "current_source",
            _ => "voltage_source",
        }
    }

    fn controlled_name(&self) -> &'static str {
        match self {
            Self::Vccs => "vccs",
            Self::Cccs => "cccs",
            Self::Ccvs => "ccvs",
            _ => "vcvs",
        }
    }

    fn analog_name(&self) -> &'static str {
        match self {
            Self::Nmos => "nmos",
            Self::Pmos => "pmos",
            Self::Npn => "npn",
            Self::Pnp => "pnp",
            Self::JfetN => "jfet_n",
            Self::JfetP => "jfet_p",
            Self::TransmissionLine => "transmission_line",
            Self::Crystal => "crystal",
            Self::Transformer => "transformer",
            Self::Varistor => "varistor",
            Self::Fuse => "fuse",
            Self::Igbt => "igbt",
            Self::Ujt => "ujt",
            Self::Put => "put",
            Self::Scr => "scr",
            Self::Triac => "triac",
            Self::Diac => "diac",
            Self::Gto => "gto",
            Self::Scs => "scs",
            Self::DiodeBridge => "diode_bridge",
            Self::DualDiode => "dual_diode",
            Self::Optocoupler => "optocoupler",
            Self::Ntc => "ntc",
            Self::Ptc => "ptc",
            Self::Rtd => "rtd",
            Self::Thermocouple => "thermocouple",
            Self::TemperatureSensor => "temperature_sensor",
            Self::ThermalSwitch => "thermal_switch",
            Self::Photoresistor => "photoresistor",
            Self::PhotodiodeSensor => "photodiode_sensor",
            Self::LinearLightSensor => "linear_light_sensor",
            Self::HallSwitch => "hall_switch",
            Self::LinearHallSensor => "linear_hall_sensor",
            Self::ReedSwitchSensor => "reed_switch_sensor",
            Self::MagnetoresistiveSensor => "magnetoresistive_sensor",
            Self::FluxgateSensor => "fluxgate_sensor",
            Self::Accelerometer => "accelerometer",
            Self::Gyroscope => "gyroscope",
            Self::Imu => "imu",
            Self::TiltSensor => "tilt_sensor",
            Self::VibrationSensor => "vibration_sensor",
            Self::StrainGaugeSensor => "strain_gauge_sensor",
            Self::LoadCell => "load_cell",
            Self::PressureSensor => "pressure_sensor",
            Self::ForceSensor => "force_sensor",
            Self::HumiditySensor => "humidity_sensor",
            Self::GasSensor => "gas_sensor",
            Self::PhSensor => "ph_sensor",
            Self::ElectrochemicalSensor => "electrochemical_sensor",
            Self::CurrentSenseResistor => "current_sense_resistor",
            Self::HallCurrentSensor => "hall_current_sensor",
            Self::VoltageDividerSensor => "voltage_divider_sensor",
            Self::InductiveProximitySensor => "inductive_proximity_sensor",
            Self::CapacitiveTouchSensor => "capacitive_touch_sensor",
            Self::UltrasonicDistanceSensor => "ultrasonic_distance_sensor",
            Self::LedActuator => "led_actuator",
            Self::DcMotor => "dc_motor",
            Self::Peltier => "peltier",
            Self::ThermoelectricGenerator => "thermoelectric_generator",
            Self::DisplayModule => "display_module",
            Self::RfAmplifier => "rf_amplifier",
            Self::DcDcConverter => "dcdc_converter",
            Self::ChargePump => "charge_pump",
            Self::VoltageReference => "voltage_reference",
            Self::CurrentReference => "current_reference",
            Self::Supervisor => "supervisor",
            Self::MotorDriver => "motor_driver",
            Self::SineVoltageSource => "sine_voltage_source",
            Self::SineCurrentSource => "sine_current_source",
            Self::Timer555 => "timer_555",
            Self::MonostableTimer => "monostable_timer",
            Self::AstableTimer => "astable_timer",
            Self::VcoTimer => "vco_timer",
            Self::PulseCurrentSource => "pulse_current_source",
            Self::ExponentialVoltageSource => "exponential_voltage_source",
            Self::PwlVoltageSource => "pwl_voltage_source",
            Self::NoiseVoltageSource => "noise_voltage_source",
            Self::NoiseCurrentSource => "noise_current_source",
            Self::BehavioralIntegrator => "behavioral_integrator",
            Self::BehavioralDifferentiator => "behavioral_differentiator",
            Self::BehavioralLowPass => "behavioral_low_pass",
            Self::BehavioralHighPass => "behavioral_high_pass",
            Self::BehavioralStateVariable => "behavioral_state_variable",
            Self::Diode => "diode",
            _ => "custom",
        }
    }

    /// Returns true when the device can conduct direct current.
    #[must_use]
    pub fn conducts_dc(&self) -> bool {
        !matches!(self, Self::Capacitor)
    }
}

/// A device instance in a structural netlist.
#[derive(Clone, Debug, PartialEq)]
pub struct DeviceInstance {
    id: DeviceId,
    name: String,
    kind: DeviceKind,
    pins: Vec<NodeId>,
    parameters: BTreeMap<String, f64>,
}

impl DeviceInstance {
    /// Creates a device instance.
    #[must_use]
    pub fn new(
        id: DeviceId,
        name: impl Into<String>,
        kind: DeviceKind,
        pins: Vec<NodeId>,
        parameters: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            pins,
            parameters,
        }
    }

    /// Returns the device identifier.
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        self.id
    }

    /// Sets or replaces a device parameter.
    pub fn set_parameter(&mut self, name: impl Into<String>, value: f64) {
        self.parameters.insert(name.into(), value);
    }
}

/// Structural view of a device.
pub trait Device {
    /// Returns the device name.
    fn name(&self) -> &str;
    /// Returns the device kind.
    fn kind(&self) -> &DeviceKind;
    /// Returns the device pins.
    fn pins(&self) -> &[NodeId];
    /// Returns the device parameters.
    fn parameters(&self) -> &BTreeMap<String, f64>;
}

impl Device for DeviceInstance {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> &DeviceKind {
        &self.kind
    }

    fn pins(&self) -> &[NodeId] {
        &self.pins
    }

    fn parameters(&self) -> &BTreeMap<String, f64> {
        &self.parameters
    }
}

/// Context for device stamping in a future analysis pass.
#[derive(Debug)]
pub struct StampContext<'a> {
    /// The MNA matrix being assembled.
    pub matrix: &'a mut DenseMatrix,
    /// The right-hand side vector.
    pub rhs: &'a mut Vec<f64>,
    /// Mapping from node identifiers to matrix rows.
    pub node_map: &'a [Option<usize>],
}

/// A solved node-voltage and branch-current vector.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solution {
    /// Node voltages indexed by node identifier.
    pub node_voltages: Vec<f64>,
    /// Branch currents indexed by MNA matrix index.
    pub branch_currents: Vec<f64>,
}

impl Solution {
    /// Creates a zero solution with the requested node slots and matrix size.
    #[must_use]
    pub fn zero(node_slots: usize, matrix_size: usize) -> Self {
        Self {
            node_voltages: vec![0.0; node_slots],
            branch_currents: vec![0.0; matrix_size],
        }
    }

    /// Returns the voltage for a node, or `None` when it is not present.
    #[must_use]
    pub fn voltage(&self, node: NodeId) -> Option<f64> {
        self.node_voltages.get(node.raw() as usize).copied()
    }

    /// Returns a branch current, or `None` when it is not present.
    #[must_use]
    pub fn branch_current(&self, index: usize) -> Option<f64> {
        self.branch_currents.get(index).copied()
    }
}
