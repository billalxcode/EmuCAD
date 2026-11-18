use super::*;

impl ModelClass {
    /// Returns a scalar smoke response for catalog checks.
    #[must_use]
    pub fn scalar_response(self, input: f64) -> f64 {
        match self.family() {
            Family::Passive | Family::Source | Family::Board => input,
            Family::Diode => diode_response(input),
            Family::Transistor => transistor_response(self, input),
            Family::Protection => protection_response(self, input),
            Family::Switch => input,
            Family::Analog => analog_response(self, input),
            Family::Logic => logic_response(self, input),
            Family::Memory => input,
            Family::Mcu => mcu_response(self, input),
            Family::Power => power_response(self, input),
            Family::Sensor => input,
            Family::Actuator => input,
            Family::Rf => input,
            Family::Behavioral => input,
        }
    }
}

fn diode_response(input: f64) -> f64 {
    let voltage = input.clamp(-40.0, 40.0);
    (voltage.exp() - 1.0).clamp(-1.0e6, 1.0e6)
}

fn transistor_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Bjt => input * 100.0,
        ModelClass::Jfet | ModelClass::Mosfet => input * input,
        _ => input,
    }
}

fn protection_response(class: ModelClass, input: f64) -> f64 {
    if class == ModelClass::Thyristor {
        return if input > 0.0 { 1.0 } else { 0.0 };
    }
    if input.abs() > 1.0 {
        return 0.0;
    }
    1.0
}

fn analog_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::OpAmp | ModelClass::Comparator => (input * 1.0e5).clamp(-1.0e6, 1.0e6),
        ModelClass::Timer => {
            if input > 0.0 {
                1.0
            } else {
                0.0
            }
        }
        _ => input.clamp(-10.0, 10.0),
    }
}

fn logic_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Counter => input.floor().clamp(0.0, 1.0e6),
        ModelClass::Adder | ModelClass::LevelShifter => input + 1.0,
        ModelClass::LogicGate | ModelClass::FlipFlop => {
            if input > 0.5 {
                1.0
            } else {
                0.0
            }
        }
        _ => input,
    }
}

fn mcu_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Adc => input.round().clamp(0.0, 4095.0),
        ModelClass::Dac => input.clamp(0.0, 4095.0),
        _ => input,
    }
}

fn power_response(class: ModelClass, input: f64) -> f64 {
    match class {
        ModelClass::Regulator => input.clamp(-20.0, 20.0),
        ModelClass::DcDc => input * 0.9,
        ModelClass::ChargePump => input * 2.0,
        ModelClass::Supervisor => {
            if input > 2.5 {
                1.0
            } else {
                0.0
            }
        }
        _ => input,
    }
}
