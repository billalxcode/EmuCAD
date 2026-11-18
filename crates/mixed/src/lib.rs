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

/// Internal state for one AVR pin link.
#[derive(Clone, Copy)]
struct AvrPinState {
    link: AvrPin,
    voltage: f64,
    level: bool,
    changed: bool,
}

/// A functional AVR CPU driving one or more analog pins.
pub struct AvrSession {
    cpu: Cpu,
    analog: TransientSession,
    pins: Vec<AvrPinState>,
}

impl AvrSession {
    /// Builds a session around one AVR program and one analog pin.
    pub fn new(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pin: AvrPin,
    ) -> Result<Self, SimError> {
        Self::with_pins(program, circuit, config, vec![pin])
    }

    /// Builds a session around one AVR program and several analog pins.
    pub fn with_pins(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pins: Vec<AvrPin>,
    ) -> Result<Self, SimError> {
        if pins.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR session requires at least one pin".to_string(),
            ));
        }
        let cpu = Cpu::new(program)?;
        let analog = TransientSession::new(circuit, config)?;
        let pins = pins
            .into_iter()
            .map(|link| AvrPinState {
                link,
                voltage: 0.0,
                level: false,
                changed: false,
            })
            .collect();
        Ok(Self { cpu, analog, pins })
    }

    /// Steps one AVR instruction and one analog step.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        self.apply_inputs()?;
        let trace = self.cpu.step()?;
        let driven = self.drive_sources()?;
        let sample = self.analog.step()?;
        self.update_pins(&driven, &sample);
        Ok(trace)
    }

    fn apply_inputs(&mut self) -> Result<(), SimError> {
        for index in 0..self.pins.len() {
            let link = self.pins[index].link;
            let external = ExternalPin::Thevenin {
                voltage: self.pins[index].voltage,
                resistance: link.external_resistance,
            };
            self.cpu
                .set_pin_voltage(link.port, link.bit, external, &link.config)?;
        }
        Ok(())
    }

    fn drive_sources(&mut self) -> Result<Vec<f64>, SimError> {
        let mut driven = Vec::with_capacity(self.pins.len());
        for index in 0..self.pins.len() {
            let link = self.pins[index].link;
            let state =
                self.cpu
                    .pin_electrical(link.port, link.bit, ExternalPin::HighZ, &link.config)?;
            self.analog.set_source_voltage(link.source, state.voltage)?;
            driven.push(state.voltage);
        }
        Ok(driven)
    }

    fn update_pins(&mut self, driven: &[f64], sample: &analysis::TransientSample) {
        for (index, value) in driven.iter().enumerate() {
            let link = self.pins[index].link;
            let voltage = sample.solution.voltage(link.node).unwrap_or(0.0);
            let level = *value >= link.config.input_threshold;
            let state = &mut self.pins[index];
            state.changed = level != state.level;
            state.level = level;
            state.voltage = voltage;
        }
    }

    /// Returns one digital pin level.
    #[must_use]
    pub fn pin_level(&self, index: usize) -> Option<bool> {
        self.pins.get(index).map(|pin| pin.level)
    }

    /// Returns true when the indexed pin changed on the last step.
    #[must_use]
    pub fn pin_changed(&self, index: usize) -> Option<bool> {
        self.pins.get(index).map(|pin| pin.changed)
    }

    /// Returns true when any pin changed on the last step.
    #[must_use]
    pub fn any_pin_changed(&self) -> bool {
        self.pins.iter().any(|pin| pin.changed)
    }

    /// Runs until any pin changes or the step limit is reached.
    pub fn run_until_pin_change(&mut self, max_steps: usize) -> Result<Option<AvrTrace>, SimError> {
        for _ in 0..max_steps {
            let trace = self.step()?;
            if self.any_pin_changed() {
                return Ok(trace);
            }
        }
        Ok(None)
    }

    /// Returns one analog node voltage.
    #[must_use]
    pub fn node_voltage(&self, index: usize) -> Option<f64> {
        self.pins.get(index).map(|pin| pin.voltage)
    }

    /// Returns the first analog node voltage.
    #[must_use]
    pub fn voltage(&self) -> f64 {
        self.pins.first().map(|pin| pin.voltage).unwrap_or(0.0)
    }

    /// Returns the AVR CPU.
    #[must_use]
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
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
        let source_id = add_source(&mut builder, "V1", source, ground);
        add_passives(
            &mut builder,
            "",
            source,
            output,
            ground,
            resistance,
            capacitance,
        );
        (builder.build().expect("circuit"), source_id, output)
    }

    fn add_source(
        builder: &mut CircuitBuilder,
        name: &str,
        source: NodeId,
        ground: NodeId,
    ) -> DeviceId {
        let params = BTreeMap::from([("voltage".to_string(), 0.0)]);
        builder
            .device(
                name,
                DeviceKind::VoltageSource,
                vec![source, ground],
                params,
            )
            .expect("source")
    }

    fn add_passives(
        builder: &mut CircuitBuilder,
        prefix: &str,
        source: NodeId,
        output: NodeId,
        ground: NodeId,
        resistance: f64,
        capacitance: f64,
    ) {
        add_resistor(builder, &format!("{prefix}R1"), source, output, resistance);
        add_capacitor(builder, &format!("{prefix}C1"), output, ground, capacitance);
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
        let source_id = add_source(&mut builder, "V1", source, ground);
        add_resistor(&mut builder, "R0", source, middle, series);
        add_resistor(&mut builder, "R1", middle, output, resistance);
        add_capacitor(&mut builder, "C1", output, ground, capacitance);
        (builder.build().expect("circuit"), source_id, output)
    }

    fn dual_rc_circuit() -> (Circuit, Vec<(DeviceId, NodeId)>) {
        let mut builder = CircuitBuilder::new();
        let source_a = builder.node("a_in").expect("node");
        let node_a = builder.node("a_out").expect("node");
        let source_b = builder.node("b_in").expect("node");
        let node_b = builder.node("b_out").expect("node");
        let ground = builder.ground();
        let a_id = add_source(&mut builder, "V1", source_a, ground);
        let b_id = add_source(&mut builder, "V2", source_b, ground);
        add_passives(&mut builder, "a_", source_a, node_a, ground, 1000.0, 1e-6);
        add_passives(&mut builder, "b_", source_b, node_b, ground, 2000.0, 1e-6);
        (
            builder.build().expect("circuit"),
            vec![(a_id, node_a), (b_id, node_b)],
        )
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
    fn avr_multiple_links_share_one_cpu() {
        let (circuit, links) = dual_rc_circuit();
        let pins = dual_pins(&links);
        let mut session =
            AvrSession::with_pins(blink_program(), &circuit, &config(1e-5, 5000), pins)
                .expect("session");
        let (high, low) = run_dual(&mut session, 5000);
        assert!(high, "analog nodes never rose");
        assert!(low, "analog nodes never fell");
        assert_eq!(
            session.pin_level(0),
            Some(session.pin_level(1).expect("pin"))
        );
    }

    fn dual_pins(links: &[(DeviceId, NodeId)]) -> Vec<AvrPin> {
        links
            .iter()
            .map(|(source, node)| AvrPin {
                port: 0,
                bit: 5,
                source: *source,
                node: *node,
                config: PinElectricalConfig::default(),
                external_resistance: 1000.0,
            })
            .collect()
    }

    fn run_dual(session: &mut AvrSession, steps: usize) -> (bool, bool) {
        let mut high = [false; 2];
        let mut low = [false; 2];
        for _ in 0..steps {
            session.step().expect("step");
            for index in 0..2 {
                let voltage = session.node_voltage(index).expect("node");
                high[index] |= voltage > 3.0;
                low[index] |= voltage < 2.0;
            }
        }
        (
            high.iter().all(|value| *value),
            low.iter().all(|value| *value),
        )
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
