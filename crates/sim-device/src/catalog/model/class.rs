use super::*;

impl ModelClass {
    /// Returns the top-level family.
    #[must_use]
    pub const fn family(self) -> Family {
        if let Some(family) = self.passive_source_family() {
            return family;
        }
        if let Some(family) = self.semiconductor_family() {
            return family;
        }
        if let Some(family) = self.digital_family() {
            return family;
        }
        self.power_family()
    }

    const fn passive_source_family(self) -> Option<Family> {
        match self {
            Self::Resistor
            | Self::Capacitor
            | Self::Inductor
            | Self::Transformer
            | Self::Crystal
            | Self::TransmissionLine => Some(Family::Passive),
            Self::VoltageSource
            | Self::CurrentSource
            | Self::Vcvs
            | Self::Vccs
            | Self::Cccs
            | Self::Ccvs
            | Self::BehavioralSource
            | Self::DigitalSource
            | Self::Battery => Some(Family::Source),
            _ => None,
        }
    }

    const fn semiconductor_family(self) -> Option<Family> {
        match self {
            Self::Diode | Self::DiodeArray | Self::Led | Self::Photodiode | Self::Optocoupler => {
                Some(Family::Diode)
            }
            Self::Bjt | Self::Jfet | Self::Mosfet | Self::OtherTransistor => {
                Some(Family::Transistor)
            }
            Self::Thyristor | Self::Protection => Some(Family::Protection),
            Self::Switch
            | Self::Relay
            | Self::SolidStateRelay
            | Self::AnalogSwitch
            | Self::Connector => Some(Family::Switch),
            Self::OpAmp | Self::Comparator | Self::Timer | Self::VoltageReference => {
                Some(Family::Analog)
            }
            _ => None,
        }
    }

    const fn digital_family(self) -> Option<Family> {
        match self {
            Self::LogicGate
            | Self::LogicGateSpecial
            | Self::FlipFlop
            | Self::Counter
            | Self::Multiplexer
            | Self::Adder
            | Self::LevelShifter
            | Self::ProgrammableLogic => Some(Family::Logic),
            Self::Memory => Some(Family::Memory),
            Self::Mcu | Self::Adc | Self::Dac | Self::SerialInterface | Self::TimerPeripheral => {
                Some(Family::Mcu)
            }
            _ => None,
        }
    }

    const fn power_family(self) -> Family {
        match self {
            Self::Regulator
            | Self::DcDc
            | Self::ChargePump
            | Self::Supervisor
            | Self::PowerSwitch
            | Self::MotorDriver => Family::Power,
            Self::Sensor => Family::Sensor,
            Self::Actuator | Self::Display => Family::Actuator,
            Self::RfBlock => Family::Rf,
            Self::BehavioralBlock => Family::Behavioral,
            _ => Family::Board,
        }
    }

    /// Returns the model fidelity tier.
    #[must_use]
    pub const fn tier(self) -> u8 {
        if self.tier_zero() {
            return 0;
        }
        if self.tier_one() {
            return 1;
        }
        if self.tier_two() {
            return 2;
        }
        4
    }

    const fn tier_zero(self) -> bool {
        matches!(
            self,
            Self::Resistor
                | Self::Capacitor
                | Self::Inductor
                | Self::VoltageSource
                | Self::CurrentSource
                | Self::Battery
        )
    }

    const fn tier_one(self) -> bool {
        self.tier_one_passive() || self.tier_one_switch()
    }

    const fn tier_one_passive(self) -> bool {
        matches!(
            self,
            Self::Transformer
                | Self::Crystal
                | Self::TransmissionLine
                | Self::Vcvs
                | Self::Vccs
                | Self::Cccs
                | Self::Ccvs
                | Self::BehavioralSource
                | Self::DigitalSource
        )
    }

    const fn tier_one_switch(self) -> bool {
        matches!(
            self,
            Self::Switch
                | Self::Relay
                | Self::SolidStateRelay
                | Self::AnalogSwitch
                | Self::Connector
                | Self::Protection
                | Self::PowerSwitch
        )
    }

    const fn tier_two(self) -> bool {
        matches!(
            self,
            Self::Diode
                | Self::DiodeArray
                | Self::Led
                | Self::Photodiode
                | Self::Optocoupler
                | Self::Bjt
                | Self::Jfet
                | Self::Mosfet
                | Self::OtherTransistor
                | Self::Thyristor
        )
    }

    /// Returns true when the class maps to a solver device kind.
    #[must_use]
    pub const fn requires_device(self) -> bool {
        self.requires_passive_device()
            || self.requires_semiconductor_device()
            || self.requires_analog_device()
            || matches!(
                self,
                Self::LogicGate | Self::SolidStateRelay | Self::Connector
            )
    }

    const fn requires_analog_device(self) -> bool {
        matches!(
            self,
            Self::OpAmp
                | Self::Comparator
                | Self::VoltageReference
                | Self::PowerSwitch
                | Self::Regulator
        )
    }

    const fn requires_passive_device(self) -> bool {
        matches!(
            self,
            Self::Resistor
                | Self::Capacitor
                | Self::Inductor
                | Self::Transformer
                | Self::Crystal
                | Self::TransmissionLine
                | Self::VoltageSource
                | Self::CurrentSource
                | Self::Vcvs
                | Self::Vccs
                | Self::Cccs
                | Self::Ccvs
                | Self::Battery
        )
    }

    const fn requires_semiconductor_device(self) -> bool {
        matches!(
            self,
            Self::Diode
                | Self::Led
                | Self::Photodiode
                | Self::Bjt
                | Self::Jfet
                | Self::Mosfet
                | Self::Switch
                | Self::Relay
                | Self::AnalogSwitch
        )
    }

    /// Returns a short working principle.
    #[must_use]
    pub const fn principle(self) -> &'static str {
        match self.family() {
            Family::Passive => self.passive_principle(),
            Family::Source => self.source_principle(),
            Family::Diode => self.diode_principle(),
            Family::Transistor => self.transistor_principle(),
            Family::Protection => self.protection_principle(),
            Family::Switch => self.switch_principle(),
            Family::Analog => self.analog_principle(),
            Family::Logic => self.logic_principle(),
            Family::Memory => "Stores state or charge and exposes read and write timing.",
            Family::Mcu => self.mcu_principle(),
            Family::Power => self.power_principle(),
            Family::Sensor => "Converts a physical quantity into an electrical signal.",
            Family::Actuator => "Converts an electrical signal into physical output.",
            Family::Rf => "Frequency-domain gain, filtering, or radiation behavior.",
            Family::Behavioral => "User-defined algebraic or table-driven transfer function.",
            Family::Board => "Board-level interconnect and module behavior.",
        }
    }

    const fn passive_principle(self) -> &'static str {
        match self {
            Self::Resistor => "Linear voltage-current relation V = I x R.",
            Self::Capacitor => "Charge integration I = C x dV/dt.",
            Self::Inductor => "Flux integration V = L x dI/dt.",
            Self::Transformer => "Coupled magnetic windings with a turns ratio.",
            Self::Crystal => "Piezoelectric resonance with series and parallel modes.",
            _ => "Distributed transmission-line behavior.",
        }
    }

    const fn source_principle(self) -> &'static str {
        match self {
            Self::VoltageSource => "Holds a defined voltage across its terminals.",
            Self::CurrentSource => "Forces a defined current through its branch.",
            Self::Vcvs => "Output voltage is proportional to an input voltage.",
            Self::Vccs => "Output current is proportional to an input voltage.",
            Self::Cccs => "Output current is proportional to a control current.",
            Self::Ccvs => "Output voltage is proportional to a control current.",
            Self::BehavioralSource => "Output follows an expression or lookup table.",
            Self::DigitalSource => "Generates a clock, pattern, or pseudo-random sequence.",
            _ => "Stores or delivers electrical energy.",
        }
    }

    const fn diode_principle(self) -> &'static str {
        match self {
            Self::Diode => "Shockley junction with forward and reverse regions.",
            Self::DiodeArray => "Multiple diodes or a bridge in one package.",
            Self::Led => "Diode with radiative recombination and forward voltage.",
            Self::Photodiode => "Photoelectric current proportional to incident light.",
            _ => "Optical isolation between input and output.",
        }
    }

    const fn transistor_principle(self) -> &'static str {
        match self {
            Self::Bjt => "Ebers-Moll or Gummel-Poon current gain.",
            Self::Jfet => "Depletion channel controlled by a gate junction.",
            Self::Mosfet => "Field-effect channel controlled by gate voltage.",
            _ => "Datasheet-level power or high-frequency transistor behavior.",
        }
    }

    const fn protection_principle(self) -> &'static str {
        match self {
            Self::Thyristor => "Four-layer latch triggered by gate current.",
            _ => "Clamps or interrupts overcurrent and overvoltage.",
        }
    }

    const fn switch_principle(self) -> &'static str {
        match self {
            Self::Switch => "Controlled two-state conductance.",
            Self::Relay => "Coil drives a changeover contact.",
            Self::SolidStateRelay => "Optical or semiconductor isolation drives the output.",
            Self::Connector => "Board-level electrical and mechanical interconnect.",
            _ => "Controlled analog signal path.",
        }
    }

    const fn analog_principle(self) -> &'static str {
        match self {
            Self::OpAmp => "High-gain differential input with feedback.",
            Self::Comparator => "Open-loop differential input with saturated output.",
            Self::Timer => "RC threshold and comparator timing.",
            _ => "Temperature-compensated voltage or current reference.",
        }
    }

    const fn logic_principle(self) -> &'static str {
        match self {
            Self::LogicGate => "Boolean function with propagation delay.",
            Self::LogicGateSpecial => "Open-drain, tri-state, or transmission-gate behavior.",
            Self::FlipFlop => "Bistable state clocked by an input event.",
            Self::Counter => "Clocked state machine with a count output.",
            Self::Multiplexer => "Selects one of several inputs or outputs.",
            Self::Adder => "Combinational arithmetic or comparison.",
            Self::LevelShifter => "Translates between voltage domains.",
            _ => "Programmable array or lookup logic.",
        }
    }

    const fn mcu_principle(self) -> &'static str {
        match self {
            Self::Mcu => "Instruction-driven core with memory and peripherals.",
            Self::Adc => "Samples and quantizes an analog input.",
            Self::Dac => "Converts a digital code into an analog output.",
            Self::SerialInterface => "Framed serial protocol with timing.",
            _ => "Counter, compare, or watchdog peripheral.",
        }
    }

    const fn power_principle(self) -> &'static str {
        match self {
            Self::Regulator => "Feedback controls a pass element.",
            Self::DcDc => "Switched inductor or capacitor energy transfer.",
            Self::ChargePump => "Capacitive charge transfer between stages.",
            Self::Supervisor => "Threshold monitor with reset or reference output.",
            Self::PowerSwitch => "Low-resistance pass element with enable control.",
            _ => "Power bridge or driver stage.",
        }
    }
}
