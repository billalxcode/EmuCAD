//! Parameter schemas for solver-backed device kinds.

use sim_core::DeviceKind;

/// How a parameter is edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterKind {
    Number,
    Boolean,
}

/// One editable model parameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterSpec {
    pub name: &'static str,
    pub unit: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub kind: ParameterKind,
}

impl ParameterSpec {
    /// Creates a numeric parameter.
    #[must_use]
    pub const fn number(
        name: &'static str,
        unit: &'static str,
        default: f64,
        min: f64,
        max: f64,
    ) -> Self {
        Self {
            name,
            unit,
            default,
            min,
            max,
            kind: ParameterKind::Number,
        }
    }

    /// Creates a boolean parameter stored as 0 or 1.
    #[must_use]
    pub const fn boolean(name: &'static str, default: bool) -> Self {
        Self {
            name,
            unit: "",
            default: if default { 1.0 } else { 0.0 },
            min: 0.0,
            max: 1.0,
            kind: ParameterKind::Boolean,
        }
    }

    /// Returns true when the value is finite and inside the range.
    #[must_use]
    pub fn is_valid(&self, value: f64) -> bool {
        value.is_finite() && value >= self.min && value <= self.max
    }
}

const RESISTOR: &[ParameterSpec] = &[ParameterSpec::number(
    "resistance",
    "ohm",
    1000.0,
    1e-9,
    1e12,
)];

const CAPACITOR: &[ParameterSpec] = &[ParameterSpec::number("capacitance", "F", 1e-6, 1e-15, 1e6)];

const INDUCTOR: &[ParameterSpec] = &[ParameterSpec::number("inductance", "H", 1e-3, 1e-15, 1e6)];

const TRANSFORMER: &[ParameterSpec] = &[
    ParameterSpec::number("turns_ratio", "ratio", 1.0, 1e-6, 1e6),
    ParameterSpec::number("inductance", "H", 1e-3, 1e-12, 1e6),
    ParameterSpec::number("coupling", "ratio", 0.99, 0.0, 1.0),
    ParameterSpec::number("rp", "ohm", 0.1, 0.0, 1e6),
    ParameterSpec::number("rs", "ohm", 0.1, 0.0, 1e6),
];

const CRYSTAL: &[ParameterSpec] = &[
    ParameterSpec::number("frequency", "Hz", 1e6, 1e-3, 1e12),
    ParameterSpec::number("capacitance", "F", 1e-14, 1e-18, 1e6),
    ParameterSpec::number("resistance", "ohm", 50.0, 1e-9, 1e12),
    ParameterSpec::number("c0", "F", 1e-12, 0.0, 1e6),
];

const TRANSMISSION_LINE: &[ParameterSpec] = &[
    ParameterSpec::number("z0", "ohm", 50.0, 1e-9, 1e12),
    ParameterSpec::number("delay", "s", 1e-9, 1e-15, 1e6),
];

const VOLTAGE_SOURCE: &[ParameterSpec] = &[ParameterSpec::number("voltage", "V", 5.0, -1e6, 1e6)];

const CURRENT_SOURCE: &[ParameterSpec] = &[ParameterSpec::number("current", "A", 1e-3, -1e6, 1e6)];

const PULSE_SOURCE: &[ParameterSpec] = &[
    ParameterSpec::number("v1", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("v2", "V", 5.0, -1e6, 1e6),
    ParameterSpec::number("delay", "s", 0.0, 0.0, 1e6),
    ParameterSpec::number("rise", "s", 1e-9, 0.0, 1e6),
    ParameterSpec::number("fall", "s", 1e-9, 0.0, 1e6),
    ParameterSpec::number("width", "s", 1e-6, 0.0, 1e6),
    ParameterSpec::number("period", "s", 2e-6, 1e-15, 1e6),
];

const VCVS: &[ParameterSpec] = &[ParameterSpec::number("gain", "ratio", 1.0, -1e12, 1e12)];

const VCCS: &[ParameterSpec] = &[ParameterSpec::number("gm", "S", 1e-3, -1e12, 1e12)];

const CCCS: &[ParameterSpec] = &[ParameterSpec::number("gain", "ratio", 1.0, -1e12, 1e12)];

const CCVS: &[ParameterSpec] = &[ParameterSpec::number("rm", "ohm", 1.0, -1e12, 1e12)];

const DIODE: &[ParameterSpec] = &[
    ParameterSpec::number("is", "A", 1e-12, 1e-30, 1e6),
    ParameterSpec::number("n", "ratio", 1.0, 1e-6, 1e3),
    ParameterSpec::number("rs", "ohm", 0.0, 0.0, 1e12),
    ParameterSpec::number("cj0", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("vj", "V", 0.7, 1e-9, 1e6),
    ParameterSpec::number("m", "ratio", 0.5, 0.0, 10.0),
];

const BJT: &[ParameterSpec] = &[
    ParameterSpec::number("is", "A", 1e-15, 1e-30, 1e6),
    ParameterSpec::number("bf", "ratio", 100.0, 1e-6, 1e9),
    ParameterSpec::number("br", "ratio", 1.0, 1e-6, 1e9),
    ParameterSpec::number("cje", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("vje", "V", 0.75, 1e-9, 1e6),
    ParameterSpec::number("mje", "ratio", 0.33, 0.0, 10.0),
    ParameterSpec::number("cjc", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("vjc", "V", 0.75, 1e-9, 1e6),
    ParameterSpec::number("mjc", "ratio", 0.33, 0.0, 10.0),
    ParameterSpec::number("tf", "s", 0.0, 0.0, 1e6),
    ParameterSpec::number("tr", "s", 0.0, 0.0, 1e6),
];

const JFET: &[ParameterSpec] = &[
    ParameterSpec::number("vto", "V", -2.0, -1e6, 1e6),
    ParameterSpec::number("beta", "A/V^2", 1e-3, 1e-15, 1e6),
    ParameterSpec::number("lambda", "1/V", 0.0, 0.0, 1e6),
    ParameterSpec::number("cgs", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("cgd", "F", 0.0, 0.0, 1e6),
];

const MOSFET: &[ParameterSpec] = &[
    ParameterSpec::number("vto", "V", 0.7, -1e6, 1e6),
    ParameterSpec::number("kp", "A/V^2", 1e-3, 1e-15, 1e6),
    ParameterSpec::number("lambda", "1/V", 0.0, 0.0, 1e6),
    ParameterSpec::number("w", "m", 1e-6, 1e-12, 1e6),
    ParameterSpec::number("l", "m", 1e-6, 1e-12, 1e6),
    ParameterSpec::number("gamma", "V^0.5", 0.0, 0.0, 1e6),
    ParameterSpec::number("phi", "V", 0.7, 1e-9, 1e6),
    ParameterSpec::number("cox", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("cgso", "F", 0.0, 0.0, 1e6),
    ParameterSpec::number("cgdo", "F", 0.0, 0.0, 1e6),
];

const VARISTOR: &[ParameterSpec] = &[
    ParameterSpec::number("clamp_voltage", "V", 6.0, 1e-9, 1e9),
    ParameterSpec::number("clamp_current", "A", 1.0, 1e-12, 1e9),
    ParameterSpec::number("alpha", "ratio", 4.0, 1.01, 1e3),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const FUSE: &[ParameterSpec] = &[
    ParameterSpec::number("resistance", "ohm", 0.1, 1e-12, 1e12),
    ParameterSpec::number("trip_current", "A", 1.0, 1e-12, 1e9),
    ParameterSpec::number("trip_time", "s", 1e-3, 1e-12, 1e9),
    ParameterSpec::boolean("reset", false),
];

const PHOTORESISTOR: &[ParameterSpec] = &[
    ParameterSpec::number("resistance_10lux", "ohm", 10000.0, 1e-9, 1e12),
    ParameterSpec::number("gamma", "ratio", 0.8, 1e-3, 1e3),
    ParameterSpec::number("series_resistance", "ohm", 10000.0, 1e-9, 1e12),
    ParameterSpec::number("illuminance", "lux", 10.0, 0.0, 1e9),
];

const PHOTODIODE_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("responsivity", "A/W", 0.5, 1e-12, 1e6),
    ParameterSpec::number("area", "m^2", 1e-6, 1e-15, 1e6),
    ParameterSpec::number("feedback_resistance", "ohm", 1e6, 1e-9, 1e12),
    ParameterSpec::number("irradiance", "W/m^2", 1.0, 0.0, 1e9),
    ParameterSpec::number("offset", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 100.0, 1e-9, 1e12),
];

const LINEAR_LIGHT_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/lux", 0.001, 1e-12, 1e6),
    ParameterSpec::number("offset", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("illuminance", "lux", 1000.0, 0.0, 1e9),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const HALL_SWITCH: &[ParameterSpec] = &[
    ParameterSpec::number("threshold", "T", 0.005, 0.0, 1e3),
    ParameterSpec::number("hysteresis", "T", 0.001, 0.0, 1e3),
    ParameterSpec::number("magnetic_field", "T", 0.0, -1e3, 1e3),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const LINEAR_HALL_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/T", 5.0, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("magnetic_field", "T", 0.0, -1e3, 1e3),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const REED_SWITCH_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("threshold", "T", 0.005, 0.0, 1e3),
    ParameterSpec::number("hysteresis", "T", 0.001, 0.0, 1e3),
    ParameterSpec::number("magnetic_field", "T", 0.0, -1e3, 1e3),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const MAGNETORESISTIVE_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("field_sensitivity", "1/T", 10.0, 0.0, 1e9),
    ParameterSpec::number("series_resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("magnetic_field", "T", 0.0, -1e3, 1e3),
];

const FLUXGATE_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/T", 1000.0, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("magnetic_field", "T", 0.0, -1e3, 1e3),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const ACCELEROMETER: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/(m/s2)", 0.050986, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("acceleration", "m/s2", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const GYROSCOPE: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/(deg/s)", 0.01, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("angular_rate", "deg/s", 0.0, -1e9, 1e9),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const IMU: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/(m/s2)", 0.050986, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("acceleration", "m/s2", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const TILT_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/deg", 0.025, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("angle", "deg", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const VIBRATION_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/(m/s2)", 0.050986, 1e-12, 1e9),
    ParameterSpec::number("offset", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("vibration", "m/s2", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
];

const NTC: &[ParameterSpec] = &[
    ParameterSpec::number("resistance", "ohm", 10000.0, 1e-9, 1e12),
    ParameterSpec::number("beta", "K", 3950.0, 1e-3, 1e6),
    ParameterSpec::number("series_resistance", "ohm", 10000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const PTC: &[ParameterSpec] = &[
    ParameterSpec::number("resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature_coefficient", "1/C", 0.0039, 0.0, 1e6),
    ParameterSpec::number("series_resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const RTD: &[ParameterSpec] = &[
    ParameterSpec::number("resistance", "ohm", 100.0, 1e-9, 1e12),
    ParameterSpec::number("alpha", "1/C", 3.9083e-3, -1e6, 1e6),
    ParameterSpec::number("beta", "1/C^2", -5.775e-7, -1e6, 1e6),
    ParameterSpec::number("c", "1/C^4", -4.183e-12, -1e6, 1e6),
    ParameterSpec::number("series_resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const THERMOCOUPLE: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/C", 41e-6, 0.0, 1e6),
    ParameterSpec::number("cold_junction", "C", 25.0, -273.0, 1e6),
    ParameterSpec::number("offset", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const TEMPERATURE_SENSOR: &[ParameterSpec] = &[
    ParameterSpec::number("sensitivity", "V/C", 0.01, 0.0, 1e6),
    ParameterSpec::number("offset", "V", 0.0, -1e6, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const THERMAL_SWITCH: &[ParameterSpec] = &[
    ParameterSpec::number("threshold", "C", 75.0, -273.0, 1e6),
    ParameterSpec::number("hysteresis", "C", 5.0, 0.0, 1e6),
    ParameterSpec::number("output_resistance", "ohm", 1000.0, 1e-9, 1e12),
    ParameterSpec::number("temperature", "C", 25.0, -273.0, 1e6),
];

const OPTOCOUPLER: &[ParameterSpec] = &[
    ParameterSpec::number("current_transfer_ratio", "ratio", 1.0, 1e-6, 1e3),
    ParameterSpec::number("forward_voltage", "V", 1.2, 1e-3, 1e6),
    ParameterSpec::number("rs", "ohm", 10.0, 1e-9, 1e12),
    ParameterSpec::number("led_leakage", "S", 1e-9, 0.0, 1e9),
    ParameterSpec::number("vce_sat", "V", 0.2, 1e-9, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const DIODE_ARRAY: &[ParameterSpec] = &[
    ParameterSpec::number("forward_voltage", "V", 0.7, 1e-3, 1e6),
    ParameterSpec::number("n", "ratio", 1.0, 1e-6, 1e3),
    ParameterSpec::number("rs", "ohm", 0.01, 0.0, 1e12),
];

const SCR: &[ParameterSpec] = &[
    ParameterSpec::number("vgt", "V", 2.0, -1e6, 1e6),
    ParameterSpec::number("holding_current", "A", 0.01, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 1.0, 1e-9, 1e12),
    ParameterSpec::number("vd", "V", 0.7, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
    ParameterSpec::number("photo_current", "A", 0.0, 0.0, 1e6),
    ParameterSpec::number("photo_trigger", "A", 1e-3, 1e-12, 1e6),
];

const TRIAC: &[ParameterSpec] = &[
    ParameterSpec::number("vgt", "V", 5.0, -1e6, 1e6),
    ParameterSpec::number("holding_current", "A", 0.01, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 1.0, 1e-9, 1e12),
    ParameterSpec::number("vd", "V", 1.0, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const DIAC: &[ParameterSpec] = &[
    ParameterSpec::number("vbo", "V", 30.0, 1e-6, 1e6),
    ParameterSpec::number("holding_current", "A", 0.01, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 10.0, 1e-9, 1e12),
    ParameterSpec::number("vd", "V", 1.0, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const GTO: &[ParameterSpec] = &[
    ParameterSpec::number("vgt", "V", 5.0, -1e6, 1e6),
    ParameterSpec::number("vgt_off", "V", -5.0, -1e6, 1e6),
    ParameterSpec::number("holding_current", "A", 0.01, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 1.0, 1e-9, 1e12),
    ParameterSpec::number("vd", "V", 0.7, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const SCS: &[ParameterSpec] = &[
    ParameterSpec::number("vgt", "V", 5.0, -1e6, 1e6),
    ParameterSpec::number("vgt_off", "V", 1.0, -1e6, 1e6),
    ParameterSpec::number("holding_current", "A", 0.01, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 1.0, 1e-9, 1e12),
    ParameterSpec::number("vd", "V", 0.7, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const IGBT: &[ParameterSpec] = &[
    ParameterSpec::number("vth", "V", 5.0, -1e6, 1e6),
    ParameterSpec::number("kp", "A/V^2", 1.0, 1e-15, 1e9),
    ParameterSpec::number("lambda", "1/V", 0.01, 0.0, 1e6),
    ParameterSpec::number("vce_sat", "V", 0.5, 1e-9, 1e6),
    ParameterSpec::number("is", "A", 1e-6, 1e-30, 1e6),
    ParameterSpec::number("n", "ratio", 1.0, 1e-6, 1e3),
    ParameterSpec::number("rs", "ohm", 0.01, 0.0, 1e12),
];

const UJT: &[ParameterSpec] = &[
    ParameterSpec::number("rb", "ohm", 5000.0, 1e-9, 1e12),
    ParameterSpec::number("eta", "ratio", 0.6, 0.0, 0.999),
    ParameterSpec::number("vd", "V", 0.7, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 10.0, 1e-9, 1e12),
    ParameterSpec::number("iv", "A", 1e-3, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const PUT: &[ParameterSpec] = &[
    ParameterSpec::number("rg", "ohm", 1e6, 1e-9, 1e15),
    ParameterSpec::number("vd", "V", 0.7, 0.0, 1e6),
    ParameterSpec::number("ron", "ohm", 10.0, 1e-9, 1e12),
    ParameterSpec::number("iv", "A", 1e-3, 0.0, 1e6),
    ParameterSpec::number("leakage", "S", 1e-9, 0.0, 1e9),
];

const SWITCH: &[ParameterSpec] = &[
    ParameterSpec::number("vth", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("ron", "ohm", 1.0, 1e-12, 1e12),
    ParameterSpec::number("roff", "ohm", 1e9, 1e-12, 1e18),
    ParameterSpec::number("width", "V", 0.1, 1e-12, 1e6),
];

const IDEAL_SWITCH: &[ParameterSpec] = &[
    ParameterSpec::number("ron", "ohm", 0.01, 1e-12, 1e12),
    ParameterSpec::number("roff", "ohm", 1e12, 1e-12, 1e18),
    ParameterSpec::boolean("closed", false),
];

const RELAY: &[ParameterSpec] = &[
    ParameterSpec::number("rcoil", "ohm", 100.0, 1e-12, 1e12),
    ParameterSpec::number("ron", "ohm", 0.1, 1e-12, 1e12),
    ParameterSpec::number("roff", "ohm", 1e9, 1e-12, 1e18),
    ParameterSpec::number("vth", "V", 2.5, -1e6, 1e6),
];

const OPAMP: &[ParameterSpec] = &[ParameterSpec::number("gain", "ratio", 1e5, 1e-6, 1e12)];

const COMPARATOR: &[ParameterSpec] = &[ParameterSpec::number("gain", "ratio", 1e4, 1e-6, 1e12)];

const REGULATOR: &[ParameterSpec] = &[ParameterSpec::number("output_voltage", "V", 5.0, 0.0, 1e6)];

const LOGIC_GATE: &[ParameterSpec] = &[
    ParameterSpec::number("vth", "V", 2.5, -1e6, 1e6),
    ParameterSpec::number("width", "V", 0.1, 1e-12, 1e6),
];

/// Returns the editable parameter schema for a solver-backed device kind.
#[must_use]
pub const fn device_parameters(device: &DeviceKind) -> &'static [ParameterSpec] {
    let passive = passive_parameters(device);
    let source = source_parameters(device);
    let semiconductor = semiconductor_parameters(device);
    let analog = analog_parameters(device);
    first_non_empty(
        first_non_empty(passive, source),
        first_non_empty(semiconductor, analog),
    )
}

const fn first_non_empty(
    first: &'static [ParameterSpec],
    second: &'static [ParameterSpec],
) -> &'static [ParameterSpec] {
    if first.is_empty() {
        return second;
    }
    first
}

const fn passive_parameters(device: &DeviceKind) -> &'static [ParameterSpec] {
    match device {
        DeviceKind::Resistor => RESISTOR,
        DeviceKind::Capacitor => CAPACITOR,
        DeviceKind::Inductor => INDUCTOR,
        DeviceKind::Transformer => TRANSFORMER,
        DeviceKind::Crystal => CRYSTAL,
        DeviceKind::TransmissionLine => TRANSMISSION_LINE,
        _ => &[],
    }
}

const fn source_parameters(device: &DeviceKind) -> &'static [ParameterSpec] {
    match device {
        DeviceKind::VoltageSource => VOLTAGE_SOURCE,
        DeviceKind::CurrentSource => CURRENT_SOURCE,
        DeviceKind::PulseVoltageSource => PULSE_SOURCE,
        DeviceKind::Vcvs => VCVS,
        DeviceKind::Vccs => VCCS,
        DeviceKind::Cccs => CCCS,
        DeviceKind::Ccvs => CCVS,
        _ => &[],
    }
}

const fn semiconductor_parameters(device: &DeviceKind) -> &'static [ParameterSpec] {
    match device {
        DeviceKind::Diode => DIODE,
        DeviceKind::Npn | DeviceKind::Pnp => BJT,
        DeviceKind::JfetN | DeviceKind::JfetP => JFET,
        DeviceKind::Nmos | DeviceKind::Pmos => MOSFET,
        DeviceKind::Switch => SWITCH,
        DeviceKind::IdealSwitch => IDEAL_SWITCH,
        DeviceKind::Relay => RELAY,
        DeviceKind::Varistor => VARISTOR,
        DeviceKind::Fuse => FUSE,
        DeviceKind::Igbt => IGBT,
        DeviceKind::Ujt => UJT,
        DeviceKind::Put => PUT,
        DeviceKind::Scr => SCR,
        DeviceKind::Triac => TRIAC,
        DeviceKind::Diac => DIAC,
        DeviceKind::Gto => GTO,
        DeviceKind::Scs => SCS,
        DeviceKind::DiodeBridge | DeviceKind::DualDiode => DIODE_ARRAY,
        DeviceKind::Optocoupler => OPTOCOUPLER,
        DeviceKind::Ntc => NTC,
        DeviceKind::Ptc => PTC,
        DeviceKind::Rtd => RTD,
        DeviceKind::Thermocouple => THERMOCOUPLE,
        DeviceKind::TemperatureSensor => TEMPERATURE_SENSOR,
        DeviceKind::ThermalSwitch => THERMAL_SWITCH,
        DeviceKind::Photoresistor => PHOTORESISTOR,
        DeviceKind::PhotodiodeSensor => PHOTODIODE_SENSOR,
        DeviceKind::LinearLightSensor => LINEAR_LIGHT_SENSOR,
        DeviceKind::HallSwitch => HALL_SWITCH,
        DeviceKind::LinearHallSensor => LINEAR_HALL_SENSOR,
        DeviceKind::ReedSwitchSensor => REED_SWITCH_SENSOR,
        DeviceKind::MagnetoresistiveSensor => MAGNETORESISTIVE_SENSOR,
        DeviceKind::FluxgateSensor => FLUXGATE_SENSOR,
        DeviceKind::Accelerometer => ACCELEROMETER,
        DeviceKind::Gyroscope => GYROSCOPE,
        DeviceKind::Imu => IMU,
        DeviceKind::TiltSensor => TILT_SENSOR,
        DeviceKind::VibrationSensor => VIBRATION_SENSOR,
        _ => &[],
    }
}

const fn analog_parameters(device: &DeviceKind) -> &'static [ParameterSpec] {
    match device {
        DeviceKind::OpAmp => OPAMP,
        DeviceKind::Comparator => COMPARATOR,
        DeviceKind::Regulator => REGULATOR,
        DeviceKind::LogicGate => LOGIC_GATE,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{ParameterKind, device_parameters};
    use sim_core::DeviceKind;

    const DEVICES: &[DeviceKind] = &[
        DeviceKind::Resistor,
        DeviceKind::Capacitor,
        DeviceKind::Inductor,
        DeviceKind::TransmissionLine,
        DeviceKind::VoltageSource,
        DeviceKind::CurrentSource,
        DeviceKind::PulseVoltageSource,
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
        DeviceKind::Varistor,
        DeviceKind::Fuse,
        DeviceKind::Igbt,
        DeviceKind::Ujt,
        DeviceKind::Put,
        DeviceKind::Scr,
        DeviceKind::Triac,
        DeviceKind::Diac,
        DeviceKind::Gto,
        DeviceKind::Scs,
        DeviceKind::DiodeBridge,
        DeviceKind::DualDiode,
        DeviceKind::Optocoupler,
        DeviceKind::Ntc,
        DeviceKind::Ptc,
        DeviceKind::Rtd,
        DeviceKind::Thermocouple,
        DeviceKind::TemperatureSensor,
        DeviceKind::ThermalSwitch,
        DeviceKind::Photoresistor,
        DeviceKind::PhotodiodeSensor,
        DeviceKind::LinearLightSensor,
        DeviceKind::HallSwitch,
        DeviceKind::LinearHallSensor,
        DeviceKind::ReedSwitchSensor,
        DeviceKind::MagnetoresistiveSensor,
        DeviceKind::FluxgateSensor,
        DeviceKind::Accelerometer,
        DeviceKind::Gyroscope,
        DeviceKind::Imu,
        DeviceKind::TiltSensor,
        DeviceKind::VibrationSensor,
    ];

    #[test]
    fn every_solver_device_has_parameters() {
        for device in DEVICES {
            assert!(!device_parameters(device).is_empty(), "{}", device.name());
        }
    }

    #[test]
    fn catalog_primary_parameters_are_in_schema() {
        for entry in crate::catalog() {
            let Some(device) = entry.device.as_ref() else {
                continue;
            };
            let schema = device_parameters(device);
            assert!(
                schema
                    .iter()
                    .any(|parameter| parameter.name == entry.parameter),
                "{} {}",
                entry.id,
                entry.parameter
            );
        }
    }

    #[test]
    fn defaults_are_inside_ranges() {
        for device in DEVICES {
            for parameter in device_parameters(device) {
                assert!(
                    parameter.is_valid(parameter.default),
                    "{} {}",
                    device.name(),
                    parameter.name
                );
            }
        }
    }

    #[test]
    fn boolean_parameters_use_zero_or_one() {
        for device in DEVICES {
            for parameter in device_parameters(device) {
                if parameter.kind != ParameterKind::Boolean {
                    continue;
                }
                assert_eq!(parameter.min, 0.0);
                assert_eq!(parameter.max, 1.0);
            }
        }
    }
}
