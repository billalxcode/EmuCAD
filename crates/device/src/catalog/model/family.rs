use super::*;

impl Family {
    /// Returns a stable family name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Passive => "passive",
            Self::Source => "source",
            Self::Diode => "diode",
            Self::Transistor => "transistor",
            Self::Protection => "protection",
            Self::Switch => "switch",
            Self::Analog => "analog",
            Self::Logic => "logic",
            Self::Memory => "memory",
            Self::Mcu => "mcu",
            Self::Power => "power",
            Self::Sensor => "sensor",
            Self::Actuator => "actuator",
            Self::Rf => "rf",
            Self::Behavioral => "behavioral",
            Self::Board => "board",
        }
    }

    /// Returns the primary reference for a family.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Self::Passive => "ngspice manual, passive devices",
            Self::Source => "ngspice manual, independent and controlled sources",
            Self::Diode => "ngspice manual, diode and optoelectronic devices",
            Self::Transistor => "ngspice manual, BJT, JFET, and MOSFET models",
            Self::Protection => "datasheet, overcurrent and overvoltage devices",
            Self::Switch => "ngspice manual, switch and relay behavior",
            Self::Analog => "datasheet and ngspice op-amp models",
            Self::Logic => "ngspice XSPICE digital models and 74xx library",
            Self::Memory => "datasheet, memory timing and retention",
            Self::Mcu => "datasheet, instruction and peripheral model",
            Self::Power => "datasheet, regulator and converter behavior",
            Self::Sensor => "datasheet, transducer transfer function",
            Self::Actuator => "datasheet, electrical to physical response",
            Self::Rf => "datasheet and S-parameter reference",
            Self::Behavioral => "user-defined transfer function",
            Self::Board => "board and module datasheet",
        }
    }
}
