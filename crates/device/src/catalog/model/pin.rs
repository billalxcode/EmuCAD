//! Electrical pin roles for solver-backed devices.

use kernel::DeviceKind;

/// Electrical role of one device pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinRole {
    Terminal1,
    Terminal2,
    Positive,
    Negative,
    Anode,
    Anode1,
    Anode2,
    Cathode,
    Cathode1,
    Cathode2,
    Collector,
    Base,
    Base1,
    Base2,
    Emitter,
    MainTerminal1,
    MainTerminal2,
    AnodeGate,
    CathodeGate,
    Gate,
    Drain,
    Source,
    OutputPositive,
    OutputNegative,
    ControlPositive,
    ControlNegative,
    SwitchA,
    SwitchB,
    CoilPositive,
    CoilNegative,
    Common,
    NormallyOpen,
    NormallyClosed,
    NonInverting,
    Inverting,
    PositiveSupply,
    NegativeSupply,
    Input1,
    Input2,
    Input,
    Output,
    Ground,
    Port1Positive,
    Port1Negative,
    Port2Positive,
    Port2Negative,
}

impl PinRole {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Terminal1 => "terminal 1",
            Self::Terminal2 => "terminal 2",
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Anode => "anode",
            Self::Anode1 => "anode 1",
            Self::Anode2 => "anode 2",
            Self::Cathode => "cathode",
            Self::Cathode1 => "cathode 1",
            Self::Cathode2 => "cathode 2",
            Self::Collector => "collector",
            Self::Base => "base",
            Self::Base1 => "base 1",
            Self::Base2 => "base 2",
            Self::Emitter => "emitter",
            Self::MainTerminal1 => "MT1",
            Self::MainTerminal2 => "MT2",
            Self::AnodeGate => "anode gate",
            Self::CathodeGate => "cathode gate",
            Self::Gate => "gate",
            Self::Drain => "drain",
            Self::Source => "source",
            Self::OutputPositive => "output +",
            Self::OutputNegative => "output -",
            Self::ControlPositive => "control +",
            Self::ControlNegative => "control -",
            Self::SwitchA => "switch A",
            Self::SwitchB => "switch B",
            Self::CoilPositive => "coil +",
            Self::CoilNegative => "coil -",
            Self::Common => "common",
            Self::NormallyOpen => "normally open",
            Self::NormallyClosed => "normally closed",
            Self::NonInverting => "non-inverting",
            Self::Inverting => "inverting",
            Self::PositiveSupply => "positive supply",
            Self::NegativeSupply => "negative supply",
            Self::Input1 => "input 1",
            Self::Input2 => "input 2",
            Self::Input => "input",
            Self::Output => "output",
            Self::Ground => "ground",
            Self::Port1Positive => "port 1 +",
            Self::Port1Negative => "port 1 -",
            Self::Port2Positive => "port 2 +",
            Self::Port2Negative => "port 2 -",
        }
    }
}

const TWO_TERMINAL: &[PinRole] = &[PinRole::Terminal1, PinRole::Terminal2];
const SOURCE: &[PinRole] = &[PinRole::Positive, PinRole::Negative];
const CONTROLLED: &[PinRole] = &[
    PinRole::OutputPositive,
    PinRole::OutputNegative,
    PinRole::ControlPositive,
    PinRole::ControlNegative,
];
const DIODE: &[PinRole] = &[PinRole::Anode, PinRole::Cathode];
const BJT: &[PinRole] = &[PinRole::Collector, PinRole::Base, PinRole::Emitter];
const JFET: &[PinRole] = &[PinRole::Drain, PinRole::Gate, PinRole::Source];
const IGBT: &[PinRole] = &[PinRole::Collector, PinRole::Gate, PinRole::Emitter];
const UJT: &[PinRole] = &[PinRole::Base1, PinRole::Base2, PinRole::Emitter];
const PUT: &[PinRole] = &[PinRole::Anode, PinRole::Gate, PinRole::Cathode];
const SCR: &[PinRole] = &[PinRole::Anode, PinRole::Gate, PinRole::Cathode];
const SENSOR: &[PinRole] = &[PinRole::PositiveSupply, PinRole::Ground, PinRole::Output];
const RF_AMPLIFIER: &[PinRole] = &[PinRole::Input, PinRole::Output, PinRole::Ground];
const DCDC_CONVERTER: &[PinRole] = &[
    PinRole::Input1,
    PinRole::Input2,
    PinRole::OutputPositive,
    PinRole::OutputNegative,
];
const CHARGE_PUMP: &[PinRole] = &[PinRole::Input1, PinRole::OutputPositive, PinRole::Ground];
const TIMER8: &[PinRole] = &[
    PinRole::PositiveSupply,
    PinRole::Ground,
    PinRole::Output,
    PinRole::Input1,
    PinRole::Input2,
    PinRole::Input,
    PinRole::ControlPositive,
    PinRole::ControlNegative,
];
const BEHAVIORAL_FILTER: &[PinRole] = &[
    PinRole::Input,
    PinRole::Output,
    PinRole::Ground,
    PinRole::Common,
];
const BEHAVIORAL_CONVERTER: &[PinRole] = &[
    PinRole::Input1,
    PinRole::Input2,
    PinRole::Output,
    PinRole::Ground,
];
const BEHAVIORAL_PWM_DAC: &[PinRole] = &[
    PinRole::Input,
    PinRole::Output,
    PinRole::PositiveSupply,
    PinRole::Ground,
];
const BEHAVIORAL_OSCILLATOR: &[PinRole] = &[
    PinRole::Input,
    PinRole::Output,
    PinRole::PositiveSupply,
    PinRole::Ground,
];
const BEHAVIORAL_MATH: &[PinRole] = &[
    PinRole::Input1,
    PinRole::Input2,
    PinRole::Output,
    PinRole::Ground,
];
const BEHAVIORAL_STATE_VARIABLE: &[PinRole] = &[
    PinRole::Input,
    PinRole::Output,
    PinRole::Ground,
    PinRole::OutputPositive,
];
const MOTOR_DRIVER: &[PinRole] = &[
    PinRole::PositiveSupply,
    PinRole::Ground,
    PinRole::Input,
    PinRole::Output,
];
const DISPLAY: &[PinRole] = &[
    PinRole::PositiveSupply,
    PinRole::Ground,
    PinRole::Input1,
    PinRole::Input2,
];
const OPTOCOUPLER: &[PinRole] = &[
    PinRole::Anode,
    PinRole::Cathode,
    PinRole::Collector,
    PinRole::Emitter,
];
const TRIAC: &[PinRole] = &[
    PinRole::MainTerminal2,
    PinRole::Gate,
    PinRole::MainTerminal1,
];
const DIAC: &[PinRole] = &[PinRole::Terminal1, PinRole::Terminal2];
const DIODE_BRIDGE: &[PinRole] = &[
    PinRole::Input1,
    PinRole::Input2,
    PinRole::OutputPositive,
    PinRole::OutputNegative,
];
const DUAL_DIODE: &[PinRole] = &[
    PinRole::Anode1,
    PinRole::Cathode1,
    PinRole::Anode2,
    PinRole::Cathode2,
];
const GTO: &[PinRole] = &[PinRole::Anode, PinRole::Gate, PinRole::Cathode];
const SCS: &[PinRole] = &[
    PinRole::Anode,
    PinRole::AnodeGate,
    PinRole::CathodeGate,
    PinRole::Cathode,
];
const MOSFET: &[PinRole] = &[PinRole::Gate, PinRole::Drain, PinRole::Source];
const SWITCH: &[PinRole] = &[
    PinRole::SwitchA,
    PinRole::SwitchB,
    PinRole::ControlPositive,
    PinRole::ControlNegative,
];
const RELAY: &[PinRole] = &[
    PinRole::CoilPositive,
    PinRole::CoilNegative,
    PinRole::Common,
    PinRole::NormallyOpen,
    PinRole::NormallyClosed,
];
const OPAMP: &[PinRole] = &[
    PinRole::NonInverting,
    PinRole::Inverting,
    PinRole::PositiveSupply,
    PinRole::NegativeSupply,
    PinRole::Output,
];
const REGULATOR: &[PinRole] = &[PinRole::Input, PinRole::Output, PinRole::Ground];
const LOGIC: &[PinRole] = &[
    PinRole::Input1,
    PinRole::Input2,
    PinRole::PositiveSupply,
    PinRole::NegativeSupply,
    PinRole::Output,
];
const CRYSTAL: &[PinRole] = &[PinRole::Terminal1, PinRole::Terminal2];
const TRANSFORMER: &[PinRole] = &[
    PinRole::Port1Positive,
    PinRole::Port1Negative,
    PinRole::Port2Positive,
    PinRole::Port2Negative,
];
const LINE: &[PinRole] = &[
    PinRole::Port1Positive,
    PinRole::Port1Negative,
    PinRole::Port2Positive,
    PinRole::Port2Negative,
];

/// Returns the pin roles for a solver-backed device.
#[must_use]
pub const fn device_pin_roles(device: &DeviceKind) -> &'static [PinRole] {
    first_non_empty(
        first_non_empty(passive_roles(device), source_roles(device)),
        first_non_empty(semiconductor_roles(device), analog_roles(device)),
    )
}

const fn first_non_empty(
    first: &'static [PinRole],
    second: &'static [PinRole],
) -> &'static [PinRole] {
    if first.is_empty() {
        return second;
    }
    first
}

const fn passive_roles(device: &DeviceKind) -> &'static [PinRole] {
    match device {
        DeviceKind::Resistor | DeviceKind::Capacitor | DeviceKind::Inductor => TWO_TERMINAL,
        DeviceKind::Crystal => CRYSTAL,
        DeviceKind::Transformer => TRANSFORMER,
        DeviceKind::TransmissionLine => LINE,
        _ => &[],
    }
}

const fn source_roles(device: &DeviceKind) -> &'static [PinRole] {
    match device {
        DeviceKind::VoltageSource | DeviceKind::PulseVoltageSource | DeviceKind::CurrentSource => {
            SOURCE
        }
        DeviceKind::Vcvs | DeviceKind::Vccs | DeviceKind::Cccs | DeviceKind::Ccvs => CONTROLLED,
        _ => &[],
    }
}

const fn semiconductor_roles(device: &DeviceKind) -> &'static [PinRole] {
    match device {
        DeviceKind::Diode | DeviceKind::LedActuator => DIODE,
        DeviceKind::Npn | DeviceKind::Pnp => BJT,
        DeviceKind::JfetN | DeviceKind::JfetP => JFET,
        DeviceKind::Nmos | DeviceKind::Pmos => MOSFET,
        DeviceKind::Igbt => IGBT,
        DeviceKind::Ujt => UJT,
        DeviceKind::Put => PUT,
        DeviceKind::Optocoupler => OPTOCOUPLER,
        DeviceKind::Ntc
        | DeviceKind::Ptc
        | DeviceKind::Rtd
        | DeviceKind::Thermocouple
        | DeviceKind::TemperatureSensor
        | DeviceKind::ThermalSwitch
        | DeviceKind::Photoresistor
        | DeviceKind::PhotodiodeSensor
        | DeviceKind::LinearLightSensor
        | DeviceKind::HallSwitch
        | DeviceKind::LinearHallSensor
        | DeviceKind::ReedSwitchSensor
        | DeviceKind::MagnetoresistiveSensor
        | DeviceKind::FluxgateSensor
        | DeviceKind::Accelerometer
        | DeviceKind::Gyroscope
        | DeviceKind::Imu
        | DeviceKind::TiltSensor
        | DeviceKind::VibrationSensor
        | DeviceKind::StrainGaugeSensor
        | DeviceKind::LoadCell
        | DeviceKind::PressureSensor
        | DeviceKind::ForceSensor
        | DeviceKind::HumiditySensor
        | DeviceKind::GasSensor
        | DeviceKind::PhSensor
        | DeviceKind::ElectrochemicalSensor
        | DeviceKind::CurrentSenseResistor
        | DeviceKind::HallCurrentSensor
        | DeviceKind::VoltageDividerSensor
        | DeviceKind::InductiveProximitySensor
        | DeviceKind::CapacitiveTouchSensor
        | DeviceKind::UltrasonicDistanceSensor => SENSOR,
        DeviceKind::Scr => SCR,
        DeviceKind::Triac => TRIAC,
        DeviceKind::Diac => DIAC,
        DeviceKind::Gto => GTO,
        DeviceKind::Scs => SCS,
        DeviceKind::DcMotor | DeviceKind::Peltier | DeviceKind::ThermoelectricGenerator => SOURCE,
        DeviceKind::DisplayModule => DISPLAY,
        DeviceKind::RfAmplifier => RF_AMPLIFIER,
        DeviceKind::DcDcConverter => DCDC_CONVERTER,
        DeviceKind::ChargePump => CHARGE_PUMP,
        DeviceKind::VoltageReference | DeviceKind::CurrentReference | DeviceKind::Supervisor => {
            SENSOR
        }
        DeviceKind::MotorDriver => MOTOR_DRIVER,
        DeviceKind::BehavioralIntegrator
        | DeviceKind::BehavioralDifferentiator
        | DeviceKind::BehavioralLowPass
        | DeviceKind::BehavioralHighPass => BEHAVIORAL_FILTER,
        DeviceKind::BehavioralStateVariable => BEHAVIORAL_STATE_VARIABLE,
        DeviceKind::BehavioralMultiplier
        | DeviceKind::BehavioralDivider
        | DeviceKind::BehavioralLog
        | DeviceKind::BehavioralAntilog
        | DeviceKind::BehavioralRms
        | DeviceKind::BehavioralComparator
        | DeviceKind::BehavioralSchmitt
        | DeviceKind::BehavioralPeakDetector
        | DeviceKind::BehavioralTrackHold
        | DeviceKind::BehavioralPrecisionRectifier => BEHAVIORAL_MATH,
        DeviceKind::BehavioralVco
        | DeviceKind::BehavioralAmplitude
        | DeviceKind::BehavioralFrequency
        | DeviceKind::BehavioralPwm => BEHAVIORAL_OSCILLATOR,
        DeviceKind::BehavioralPll => BEHAVIORAL_OSCILLATOR,
        DeviceKind::BehavioralSampleHold
        | DeviceKind::BehavioralSarAdc
        | DeviceKind::BehavioralDeltaSigmaAdc => BEHAVIORAL_CONVERTER,
        DeviceKind::BehavioralPwmDac => BEHAVIORAL_PWM_DAC,
        DeviceKind::Timer555
        | DeviceKind::MonostableTimer
        | DeviceKind::AstableTimer
        | DeviceKind::VcoTimer
        | DeviceKind::TimerPll => TIMER8,
        DeviceKind::SineVoltageSource
        | DeviceKind::SineCurrentSource
        | DeviceKind::PulseCurrentSource
        | DeviceKind::ExponentialVoltageSource
        | DeviceKind::PwlVoltageSource
        | DeviceKind::NoiseVoltageSource
        | DeviceKind::NoiseCurrentSource
        | DeviceKind::PolynomialVoltageSource => SOURCE,
        DeviceKind::DiodeBridge => DIODE_BRIDGE,
        DeviceKind::DualDiode => DUAL_DIODE,
        DeviceKind::Switch => SWITCH,
        DeviceKind::IdealSwitch | DeviceKind::Fuse | DeviceKind::Varistor => TWO_TERMINAL,
        DeviceKind::Relay => RELAY,
        _ => &[],
    }
}

const fn analog_roles(device: &DeviceKind) -> &'static [PinRole] {
    match device {
        DeviceKind::OpAmp | DeviceKind::Comparator => OPAMP,
        DeviceKind::Regulator => REGULATOR,
        DeviceKind::LogicGate => LOGIC,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{PinRole, device_pin_roles};
    use kernel::DeviceKind;

    const DEVICES: &[DeviceKind] = &[
        DeviceKind::Resistor,
        DeviceKind::Capacitor,
        DeviceKind::Inductor,
        DeviceKind::TransmissionLine,
        DeviceKind::VoltageSource,
        DeviceKind::PulseVoltageSource,
        DeviceKind::CurrentSource,
        DeviceKind::Vcvs,
        DeviceKind::Vccs,
        DeviceKind::Cccs,
        DeviceKind::Ccvs,
        DeviceKind::Diode,
        DeviceKind::Npn,
        DeviceKind::Pnp,
        DeviceKind::JfetN,
        DeviceKind::JfetP,
        DeviceKind::Nmos,
        DeviceKind::Pmos,
        DeviceKind::Switch,
        DeviceKind::IdealSwitch,
        DeviceKind::Relay,
        DeviceKind::OpAmp,
        DeviceKind::Comparator,
        DeviceKind::Regulator,
        DeviceKind::LogicGate,
    ];

    #[test]
    fn every_solver_device_has_pin_roles() {
        for device in DEVICES {
            assert!(!device_pin_roles(device).is_empty(), "{}", device.name());
        }
    }

    #[test]
    fn catalog_pin_counts_match_roles() {
        for entry in crate::catalog() {
            let Some(device) = entry.device.as_ref() else {
                continue;
            };
            assert_eq!(
                usize::from(entry.pins),
                device_pin_roles(device).len(),
                "{}",
                entry.id
            );
        }
    }

    #[test]
    fn role_labels_are_ascii() {
        for device in DEVICES {
            for role in device_pin_roles(device) {
                assert!(role.label().is_ascii(), "{}", role.label());
            }
        }
        assert_eq!(PinRole::Anode.label(), "anode");
    }
}
