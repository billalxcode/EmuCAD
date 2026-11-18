//! Deterministic digital to analog co-simulation with analog feedback.

#![forbid(unsafe_code)]

use analysis::{TransientConfig, TransientSession};
use avr::{AvrTrace, Cpu, ExternalPin, PinElectricalConfig};
use cells::catalog_digital::DigitalEntry;
use kernel::{Circuit, DeviceId, NodeId, SimError, Time};

/// Analog input fed back to one digital input.
#[derive(Clone, Copy)]
pub struct PinInput {
    /// Digital input name.
    pub name: &'static str,
    /// Analog voltage threshold.
    pub threshold: f64,
}

/// Digital output and analog source link.
#[derive(Clone, Copy)]
pub struct PinLink {
    /// Digital output name.
    pub output: &'static str,
    /// Analog voltage source device.
    pub source: DeviceId,
    /// Analog node returned to the caller.
    pub node: NodeId,
    /// Source voltage for digital high.
    pub high: f64,
    /// Source voltage for digital low.
    pub low: f64,
    /// Optional analog feedback input.
    pub input: Option<PinInput>,
}

/// A digital circuit driving an analog voltage source.
pub struct MixedSession {
    digital: DigitalEntry,
    analog: TransientSession,
    link: PinLink,
    voltage: f64,
}

impl MixedSession {
    /// Builds a session with a digital entry and a pin link.
    pub fn new(
        circuit: &Circuit,
        config: &TransientConfig,
        digital: DigitalEntry,
        link: PinLink,
    ) -> Result<Self, SimError> {
        if digital.circuit.output(link.output).is_none() {
            return Err(SimError::InvalidNetlist(format!(
                "unknown digital output: {}",
                link.output
            )));
        }
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self {
            digital,
            analog,
            link,
            voltage: 0.0,
        })
    }

    /// Advances one digital tick and one analog step, returning the node voltage.
    pub fn step(&mut self) -> Result<f64, SimError> {
        self.apply_input()?;
        self.digital.circuit.tick()?;
        let value = self.source_value()?;
        self.analog.set_source_voltage(self.link.source, value)?;
        let sample = self.analog.step()?;
        self.voltage = sample.solution.voltage(self.link.node).unwrap_or(0.0);
        Ok(self.voltage)
    }

    fn apply_input(&mut self) -> Result<(), SimError> {
        let Some(input) = self.link.input else {
            return Ok(());
        };
        let value = self.voltage > input.threshold;
        self.digital.circuit.set_input(input.name, value)
    }

    fn source_value(&self) -> Result<f64, SimError> {
        let output = self
            .digital
            .circuit
            .output(self.link.output)
            .ok_or_else(|| {
                SimError::InvalidNetlist(format!("unknown digital output: {}", self.link.output))
            })?;
        Ok(if output {
            self.link.high
        } else {
            self.link.low
        })
    }

    /// Returns the analog simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.analog.time()
    }

    /// Returns the last analog node voltage.
    #[must_use]
    pub fn voltage(&self) -> f64 {
        self.voltage
    }

    /// Returns the current digital output.
    #[must_use]
    pub fn digital_output(&self) -> Option<bool> {
        self.digital.circuit.output(self.link.output)
    }
}

/// One AVR pin linked to an analog source and node.
#[derive(Clone, Copy)]
pub struct AvrPin {
    /// AVR port index: 0 for B, 1 for C, 2 for D.
    pub port: u8,
    /// Bit index within the port.
    pub bit: u8,
    /// Analog voltage source driven by the pin.
    pub source: DeviceId,
    /// Analog node returned to the caller.
    pub node: NodeId,
    /// AVR pin electrical configuration.
    pub config: PinElectricalConfig,
    /// External Thevenin resistance seen by the pin.
    pub external_resistance: f64,
}

/// A functional AVR CPU driving one analog pin.
pub struct AvrSession {
    cpu: Cpu,
    analog: TransientSession,
    pin: AvrPin,
    voltage: f64,
    level: bool,
    pin_changed: bool,
}

impl AvrSession {
    /// Builds a session around one AVR program and analog pin.
    pub fn new(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pin: AvrPin,
    ) -> Result<Self, SimError> {
        let cpu = Cpu::new(program)?;
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self {
            cpu,
            analog,
            pin,
            voltage: 0.0,
            level: false,
            pin_changed: false,
        })
    }

    /// Steps one AVR instruction and one analog step.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        let external = ExternalPin::Thevenin {
            voltage: self.voltage,
            resistance: self.pin.external_resistance,
        };
        self.cpu
            .set_pin_voltage(self.pin.port, self.pin.bit, external, &self.pin.config)?;
        let trace = self.cpu.step()?;
        let state = self.cpu.pin_electrical(
            self.pin.port,
            self.pin.bit,
            ExternalPin::HighZ,
            &self.pin.config,
        )?;
        self.analog
            .set_source_voltage(self.pin.source, state.voltage)?;
        let sample = self.analog.step()?;
        self.voltage = sample.solution.voltage(self.pin.node).unwrap_or(0.0);
        self.update_pin(state.voltage);
        Ok(trace)
    }

    fn update_pin(&mut self, voltage: f64) {
        let level = voltage >= self.pin.config.input_threshold;
        self.pin_changed = level != self.level;
        self.level = level;
    }

    /// Returns the current digital pin level.
    #[must_use]
    pub fn pin_level(&self) -> bool {
        self.level
    }

    /// Returns true when the last step changed the pin level.
    #[must_use]
    pub fn pin_changed(&self) -> bool {
        self.pin_changed
    }

    /// Runs until the pin changes or the step limit is reached.
    pub fn run_until_pin_change(&mut self, max_steps: usize) -> Result<Option<AvrTrace>, SimError> {
        for _ in 0..max_steps {
            let trace = self.step()?;
            if self.pin_changed {
                return Ok(trace);
            }
        }
        Ok(None)
    }

    /// Returns the AVR CPU.
    #[must_use]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Returns the last analog node voltage.
    #[must_use]
    pub fn voltage(&self) -> f64 {
        self.voltage
    }

    /// Returns the analog simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.analog.time()
    }
}

#[cfg(test)]
mod tests {
    use super::{AvrPin, AvrSession, MixedSession, PinInput, PinLink};
    use analysis::TransientConfig;
    use avr::PinElectricalConfig;
    use cells::DigitalBuilder;
    use cells::catalog_digital::{DigitalEntry, digital_entry};
    use kernel::{Circuit, CircuitBuilder, DeviceId, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn rc_circuit(resistance: f64, capacitance: f64) -> (Circuit, DeviceId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let source = builder.node("in").expect("node");
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        let source_id = add_source(&mut builder, source, ground);
        add_passives(
            &mut builder,
            source,
            output,
            ground,
            resistance,
            capacitance,
        );
        (builder.build().expect("circuit"), source_id, output)
    }

    fn add_source(builder: &mut CircuitBuilder, source: NodeId, ground: NodeId) -> DeviceId {
        let params = BTreeMap::from([("voltage".to_string(), 0.0)]);
        builder
            .device(
                "V1",
                DeviceKind::VoltageSource,
                vec![source, ground],
                params,
            )
            .expect("V1")
    }

    fn add_passives(
        builder: &mut CircuitBuilder,
        source: NodeId,
        output: NodeId,
        ground: NodeId,
        resistance: f64,
        capacitance: f64,
    ) {
        add_resistor(builder, "R1", source, output, resistance);
        add_capacitor(builder, "C1", output, ground, capacitance);
    }

    fn add_resistor(
        builder: &mut CircuitBuilder,
        name: &str,
        first: NodeId,
        second: NodeId,
        resistance: f64,
    ) {
        let params = BTreeMap::from([("resistance".to_string(), resistance)]);
        builder
            .device(name, DeviceKind::Resistor, vec![first, second], params)
            .expect("resistor");
    }

    fn add_capacitor(
        builder: &mut CircuitBuilder,
        name: &str,
        first: NodeId,
        second: NodeId,
        capacitance: f64,
    ) {
        let params = BTreeMap::from([("capacitance".to_string(), capacitance)]);
        builder
            .device(name, DeviceKind::Capacitor, vec![first, second], params)
            .expect("capacitor");
    }

    fn thevenin_rc_circuit(
        series: f64,
        resistance: f64,
        capacitance: f64,
    ) -> (Circuit, DeviceId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let source = builder.node("in").expect("node");
        let middle = builder.node("mid").expect("node");
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        let source_id = add_source(&mut builder, source, ground);
        add_resistor(&mut builder, "R0", source, middle, series);
        add_resistor(&mut builder, "R1", middle, output, resistance);
        add_capacitor(&mut builder, "C1", output, ground, capacitance);
        (builder.build().expect("circuit"), source_id, output)
    }

    fn clock_entry() -> DigitalEntry {
        digital_entry("source.digital.digital_clock_source")
            .expect("build")
            .expect("clock")
    }

    fn inverter_entry() -> DigitalEntry {
        let mut builder = DigitalBuilder::new();
        let input = builder.input("a");
        let output = builder.not(input);
        builder.output("y", output);
        DigitalEntry {
            id: "test.inverter",
            circuit: builder.build(),
            inputs: vec!["a"],
            outputs: vec!["y"],
        }
    }

    fn config(dt: f64, steps: usize) -> TransientConfig {
        TransientConfig {
            dt,
            steps,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        }
    }

    #[test]
    fn clock_toggles_source_and_preserves_capacitor_state() {
        let (circuit, source, node) = rc_circuit(100.0, 1e-9);
        let link = PinLink {
            output: "q",
            source,
            node,
            high: 5.0,
            low: 0.0,
            input: None,
        };
        let mut session =
            MixedSession::new(&circuit, &config(1e-6, 10), clock_entry(), link).expect("session");
        let high = session.step().expect("high step");
        assert!(high > 4.5, "high={high}");
        let low = session.step().expect("low step");
        assert!(low < 0.5, "low={low}");
        assert_eq!(session.time().as_ps(), 2_000_000);
    }

    #[test]
    fn analog_feedback_drives_digital_inverter() {
        let (circuit, source, node) = rc_circuit(1000.0, 1e-6);
        let link = PinLink {
            output: "y",
            source,
            node,
            high: 5.0,
            low: 0.0,
            input: Some(PinInput {
                name: "a",
                threshold: 2.5,
            }),
        };
        let mut session = MixedSession::new(&circuit, &config(1e-5, 1000), inverter_entry(), link)
            .expect("session");
        let toggles = count_toggles(&mut session, 1000);
        assert!(toggles >= 4, "toggles={toggles}");
    }

    fn blink_program() -> Vec<u16> {
        vec![
            0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
        ]
    }

    #[test]
    fn avr_pin_drives_thevenin_rc() {
        let (circuit, source, node) = thevenin_rc_circuit(45.0, 1000.0, 1e-6);
        let pin = AvrPin {
            port: 0,
            bit: 5,
            source,
            node,
            config: PinElectricalConfig::default(),
            external_resistance: 1000.0,
        };
        let mut session =
            AvrSession::new(blink_program(), &circuit, &config(1e-5, 5000), pin).expect("session");
        let (high, low, voltage_high, voltage_low) = run_avr(&mut session, 5000);
        assert!(high, "PB5 never went high");
        assert!(low, "PB5 never went low");
        assert!(voltage_high, "analog node never rose");
        assert!(voltage_low, "analog node never fell");
    }

    fn run_avr(session: &mut AvrSession, steps: usize) -> (bool, bool, bool, bool) {
        let mut high = false;
        let mut low = false;
        let mut voltage_high = false;
        let mut voltage_low = false;
        for _ in 0..steps {
            session.step().expect("step");
            let portb = session.cpu().io(0x05);
            high |= portb & 0x20 != 0;
            low |= portb & 0x20 == 0;
            let voltage = session.voltage();
            voltage_high |= voltage > 3.0;
            voltage_low |= voltage < 2.0;
        }
        (high, low, voltage_high, voltage_low)
    }

    #[test]
    fn avr_pin_change_breakpoint_stops() {
        let (circuit, source, node) = thevenin_rc_circuit(45.0, 1000.0, 1e-6);
        let pin = AvrPin {
            port: 0,
            bit: 5,
            source,
            node,
            config: PinElectricalConfig::default(),
            external_resistance: 1000.0,
        };
        let mut session = AvrSession::new(blink_program(), &circuit, &config(1e-5, 10_000), pin)
            .expect("session");
        assert!(session.run_until_pin_change(5000).expect("run").is_some());
        assert!(session.run_until_pin_change(5000).expect("run").is_some());
    }

    fn count_toggles(session: &mut MixedSession, steps: usize) -> usize {
        let mut toggles = 0;
        let mut previous = session.digital_output();
        for _ in 0..steps {
            let voltage = session.step().expect("step");
            assert!((-0.1..=5.1).contains(&voltage), "voltage={voltage}");
            let current = session.digital_output();
            if current != previous {
                toggles += 1;
                previous = current;
            }
        }
        toggles
    }
}
