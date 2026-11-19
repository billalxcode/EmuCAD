//! Deterministic digital to analog co-simulation with analog feedback.

#![forbid(unsafe_code)]

use std::collections::VecDeque;

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

/// Analog node sampled by the AVR ADC.
#[derive(Clone, Copy)]
pub struct AdcLink {
    /// ADC channel index.
    pub channel: u8,
    /// Analog node sampled by the channel.
    pub node: NodeId,
    /// ADC reference voltage.
    pub reference: f64,
}

/// AVR Timer0 PWM output linked to an analog source.
#[derive(Clone, Copy)]
pub struct PwmLink {
    /// Analog voltage source driven by PWM.
    pub source: DeviceId,
    /// Analog node observed after each step.
    pub node: NodeId,
    /// Source voltage when PWM is high.
    pub high: f64,
    /// Source voltage when PWM is low.
    pub low: f64,
}

/// External byte stream attached to the AVR USART.
#[derive(Clone, Debug, Default)]
pub struct UartLink {
    rx: VecDeque<u8>,
    tx: VecDeque<u8>,
}

/// Timer0 event observed by one co-simulation step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer0Event {
    /// The counter wrapped from 0xFF to 0x00.
    Overflow,
    /// The counter matched the compare value.
    Compare,
}

/// Last-step Timer0 flags and event counter snapshots.
#[derive(Clone, Copy, Default)]
struct Timer0EventState {
    overflow: bool,
    compare: bool,
    overflow_count: u32,
    compare_count: u32,
}

/// One deterministic co-simulation event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoSimEvent {
    /// CPU cycles completed when the event was observed.
    pub cycles: u64,
    /// Analog time when the event was observed.
    pub time: Time,
    /// Event payload.
    pub kind: CoSimEventKind,
}

/// Kind of event recorded by an AVR session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoSimEventKind {
    /// One AVR pin changed level.
    PinChanged {
        /// Index in the session pin list.
        index: usize,
        /// New digital level.
        level: bool,
    },
    /// Timer0 wrapped from 0xFF to 0x00.
    Timer0Overflow,
    /// Timer0 matched its compare value.
    Timer0Compare,
    /// A UART frame completed with one byte.
    UartTx(u8),
}

impl UartLink {
    /// Builds an empty UART link.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues one byte for the AVR receiver.
    pub fn push_rx(&mut self, value: u8) {
        self.rx.push_back(value);
    }

    /// Takes one byte transmitted by the AVR.
    pub fn take_tx(&mut self) -> Option<u8> {
        self.tx.pop_front()
    }

    /// Returns the number of bytes waiting for the AVR receiver.
    #[must_use]
    pub fn rx_len(&self) -> usize {
        self.rx.len()
    }

    /// Returns the number of bytes transmitted by the AVR.
    #[must_use]
    pub fn tx_len(&self) -> usize {
        self.tx.len()
    }
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
    adc: Option<AdcLink>,
    pwm: Option<PwmLink>,
    uart: Option<UartLink>,
    adc_voltage: f64,
    pwm_voltage: f64,
    timer0_events: Timer0EventState,
    events: VecDeque<CoSimEvent>,
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
        Self::with_links(program, circuit, config, pins, None, None)
    }

    /// Builds a session with pin, ADC, and PWM links.
    pub fn with_links(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pins: Vec<AvrPin>,
        adc: Option<AdcLink>,
        pwm: Option<PwmLink>,
    ) -> Result<Self, SimError> {
        validate_links(&pins, adc, pwm)?;
        let cpu = Cpu::new(program)?;
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self::assemble(cpu, analog, pins, adc, pwm))
    }

    fn assemble(
        cpu: Cpu,
        analog: TransientSession,
        pins: Vec<AvrPin>,
        adc: Option<AdcLink>,
        pwm: Option<PwmLink>,
    ) -> Self {
        Self {
            cpu,
            analog,
            pins: pin_states(pins),
            adc,
            pwm,
            uart: None,
            adc_voltage: 0.0,
            pwm_voltage: 0.0,
            timer0_events: Timer0EventState::default(),
            events: VecDeque::new(),
        }
    }

    /// Attaches an external byte stream to the AVR USART.
    #[must_use]
    pub fn with_uart(mut self, link: UartLink) -> Self {
        self.uart = Some(link);
        self
    }

    /// Steps one AVR instruction and one analog step.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        self.apply_adc();
        self.apply_uart_rx();
        self.apply_inputs()?;
        let trace = self.cpu.step()?;
        self.update_timer0_events();
        self.record_timer0_events();
        self.drain_uart_tx();
        let driven = self.drive_sources()?;
        self.drive_pwm()?;
        let sample = self.analog.step()?;
        self.update_pins(&driven, &sample);
        self.update_adc(&sample);
        self.update_pwm(&sample);
        Ok(trace)
    }

    fn apply_uart_rx(&mut self) {
        let Some(value) = self.pending_uart_rx() else {
            return;
        };
        if self.cpu.uart().rx_ready() {
            return;
        }
        self.cpu.uart().receive(value);
        if let Some(link) = self.uart.as_mut() {
            link.rx.pop_front();
        }
    }

    fn pending_uart_rx(&self) -> Option<u8> {
        self.uart.as_ref().and_then(|link| link.rx.front().copied())
    }

    fn drain_uart_tx(&mut self) {
        if self.uart.is_none() {
            return;
        }
        while let Some(value) = self.cpu.uart().take_tx_byte() {
            if let Some(link) = self.uart.as_mut() {
                link.tx.push_back(value);
            }
            self.push_event(CoSimEventKind::UartTx(value));
        }
    }

    fn record_timer0_events(&mut self) {
        if self.timer0_events.overflow {
            self.push_event(CoSimEventKind::Timer0Overflow);
        }
        if self.timer0_events.compare {
            self.push_event(CoSimEventKind::Timer0Compare);
        }
    }

    fn push_event(&mut self, kind: CoSimEventKind) {
        self.events.push_back(CoSimEvent {
            cycles: self.cpu.cycles(),
            time: self.analog.time(),
            kind,
        });
    }

    fn update_timer0_events(&mut self) {
        let (overflow, compare) = {
            let timer = self.cpu.timer0();
            (timer.overflow_count(), timer.compare_count())
        };
        let events = &mut self.timer0_events;
        events.overflow = overflow.wrapping_sub(events.overflow_count) != 0;
        events.compare = compare.wrapping_sub(events.compare_count) != 0;
        events.overflow_count = overflow;
        events.compare_count = compare;
    }

    /// Returns true when Timer0 overflowed on the last step.
    #[must_use]
    pub fn timer0_overflow(&self) -> bool {
        self.timer0_events.overflow
    }

    /// Returns true when Timer0 matched its compare value on the last step.
    #[must_use]
    pub fn timer0_compare(&self) -> bool {
        self.timer0_events.compare
    }

    /// Takes the oldest queued co-simulation event.
    pub fn take_event(&mut self) -> Option<CoSimEvent> {
        self.events.pop_front()
    }

    /// Returns the number of queued co-simulation events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Runs until the requested Timer0 event or the step limit is reached.
    pub fn run_until_timer0_event(
        &mut self,
        event: Timer0Event,
        max_steps: usize,
    ) -> Result<Option<AvrTrace>, SimError> {
        for _ in 0..max_steps {
            let trace = self.step()?;
            if self.timer0_event(event) {
                return Ok(trace);
            }
        }
        Ok(None)
    }

    fn timer0_event(&self, event: Timer0Event) -> bool {
        match event {
            Timer0Event::Overflow => self.timer0_events.overflow,
            Timer0Event::Compare => self.timer0_events.compare,
        }
    }

    fn apply_adc(&mut self) {
        let Some(adc) = self.adc else {
            return;
        };
        let code = adc_code(self.adc_voltage, adc.reference);
        self.cpu.adc().set_input(adc.channel, code);
    }

    fn drive_pwm(&mut self) -> Result<(), SimError> {
        let Some(pwm) = self.pwm else {
            return Ok(());
        };
        let value = if self.cpu.pwm_a() { pwm.high } else { pwm.low };
        self.analog.set_source_voltage(pwm.source, value)
    }

    fn update_adc(&mut self, sample: &analysis::TransientSample) {
        let Some(adc) = self.adc else {
            return;
        };
        self.adc_voltage = sample.solution.voltage(adc.node).unwrap_or(0.0);
    }

    fn update_pwm(&mut self, sample: &analysis::TransientSample) {
        let Some(pwm) = self.pwm else {
            return;
        };
        self.pwm_voltage = sample.solution.voltage(pwm.node).unwrap_or(0.0);
    }

    /// Returns the last ADC node voltage.
    #[must_use]
    pub fn adc_voltage(&self) -> Option<f64> {
        self.adc.map(|_| self.adc_voltage)
    }

    /// Returns the last PWM node voltage.
    #[must_use]
    pub fn pwm_voltage(&self) -> Option<f64> {
        self.pwm.map(|_| self.pwm_voltage)
    }

    /// Returns the attached UART link.
    #[must_use]
    pub fn uart_link(&self) -> Option<&UartLink> {
        self.uart.as_ref()
    }

    /// Returns the attached UART link for external byte access.
    pub fn uart_link_mut(&mut self) -> Option<&mut UartLink> {
        self.uart.as_mut()
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
            let changed = level != state.level;
            state.changed = changed;
            state.level = level;
            state.voltage = voltage;
            if changed {
                self.push_event(CoSimEventKind::PinChanged { index, level });
            }
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

    /// Returns the AVR CPU for peripheral setup.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Returns the analog simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.analog.time()
    }
}

fn adc_code(voltage: f64, reference: f64) -> u16 {
    if reference <= 0.0 {
        return 0;
    }
    let ratio = (voltage / reference).clamp(0.0, 1.0);
    (ratio * 1023.0).round() as u16
}

fn validate_links(
    pins: &[AvrPin],
    adc: Option<AdcLink>,
    pwm: Option<PwmLink>,
) -> Result<(), SimError> {
    if pins.is_empty() && adc.is_none() && pwm.is_none() {
        return Err(SimError::InvalidNetlist(
            "AVR session requires at least one link".to_string(),
        ));
    }
    Ok(())
}

fn pin_states(pins: Vec<AvrPin>) -> Vec<AvrPinState> {
    pins.into_iter()
        .map(|link| AvrPinState {
            link,
            voltage: 0.0,
            level: false,
            changed: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        AdcLink, AvrPin, AvrSession, CoSimEventKind, MixedSession, PinInput, PinLink, PwmLink,
        Timer0Event, UartLink,
    };
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
        let source_id = add_source(&mut builder, "V1", source, ground, 0.0);
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
        voltage: f64,
    ) -> DeviceId {
        let params = BTreeMap::from([("voltage".to_string(), voltage)]);
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
        let source_id = add_source(&mut builder, "V1", source, ground, 0.0);
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
        let a_id = add_source(&mut builder, "V1", source_a, ground, 0.0);
        let b_id = add_source(&mut builder, "V2", source_b, ground, 0.0);
        add_passives(&mut builder, "a_", source_a, node_a, ground, 1000.0, 1e-6);
        add_passives(&mut builder, "b_", source_b, node_b, ground, 2000.0, 1e-6);
        (
            builder.build().expect("circuit"),
            vec![(a_id, node_a), (b_id, node_b)],
        )
    }

    fn fixed_circuit(voltage: f64) -> (Circuit, NodeId) {
        let mut builder = CircuitBuilder::new();
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        add_source(&mut builder, "V1", output, ground, voltage);
        (builder.build().expect("circuit"), output)
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

    #[test]
    fn analog_node_feeds_avr_adc() {
        let (circuit, node) = fixed_circuit(2.5);
        let adc = AdcLink {
            channel: 0,
            node,
            reference: 5.0,
        };
        let mut session = adc_session(circuit, adc);
        session.step().expect("settle");
        assert!((session.adc_voltage().unwrap_or(0.0) - 2.5).abs() < 0.01);
        let code = convert_adc(&mut session, 0);
        assert!((500..=520).contains(&code), "code={code}");
    }

    fn adc_session(circuit: Circuit, adc: AdcLink) -> AvrSession {
        AvrSession::with_links(
            vec![0xCFFF],
            &circuit,
            &config(1e-5, 50),
            Vec::new(),
            Some(adc),
            None,
        )
        .expect("session")
    }

    fn convert_adc(session: &mut AvrSession, channel: u8) -> u16 {
        session.cpu_mut().adc().write_control_a(channel);
        session.cpu_mut().adc().write_control_b(0xC0);
        for _ in 0..40 {
            session.step().expect("step");
        }
        let adc = session.cpu_mut().adc();
        assert!(adc.flag());
        assert!(!adc.busy());
        u16::from(adc.read_data_low()) | (u16::from(adc.read_data_high()) << 8)
    }

    #[test]
    fn uart_transmit_reaches_external_link() {
        let mut session = uart_session(vec![0xCFFF]);
        session.cpu_mut().uart().write_control(1, 0x08);
        session.cpu_mut().uart().write_tx(0x41);
        for _ in 0..100 {
            session.step().expect("step");
        }
        let link = session.uart_link_mut().expect("uart link");
        assert_eq!(link.tx_len(), 1);
        assert_eq!(link.take_tx(), Some(0x41));
        assert_eq!(link.tx_len(), 0);
        assert_eq!(link.take_tx(), None);
        let event = session.take_event().expect("uart event");
        assert_eq!(event.kind, CoSimEventKind::UartTx(0x41));
        assert_eq!(event.cycles, 160);
    }

    #[test]
    fn uart_receive_reaches_avr_register() {
        let mut session = uart_session(vec![0x9100, 0x00C6, 0x9110, 0x00C6, 0xCFFF]);
        let link = session.uart_link_mut().expect("uart link");
        link.push_rx(0x5A);
        link.push_rx(0xC3);
        assert_eq!(link.rx_len(), 2);
        session.step().expect("first byte");
        session.step().expect("second byte");
        assert_eq!(session.cpu().register(16), 0x5A);
        assert_eq!(session.cpu().register(17), 0xC3);
        assert!(!session.cpu_mut().uart().rx_ready());
        assert_eq!(session.uart_link().expect("uart link").rx_len(), 0);
    }

    fn pin_session(program: Vec<u16>) -> AvrSession {
        let (circuit, source, node) = thevenin_rc_circuit(45.0, 1000.0, 1e-6);
        let pin = AvrPin {
            port: 0,
            bit: 5,
            source,
            node,
            config: PinElectricalConfig::default(),
            external_resistance: 1000.0,
        };
        AvrSession::new(program, &circuit, &config(1e-5, 1000), pin).expect("session")
    }

    fn uart_session(program: Vec<u16>) -> AvrSession {
        pin_session(program).with_uart(UartLink::new())
    }

    #[test]
    fn timer0_compare_event_stops_session() {
        let mut session = pin_session(vec![0xCFFF]);
        configure_timer0(&mut session, 4);
        session.cpu_mut().timer0().set_ctc(true);
        let first = session.run_until_timer0_event(Timer0Event::Compare, 10);
        assert!(first.expect("run").is_some());
        assert_eq!(session.cpu().cycles(), 4);
        let second = session.run_until_timer0_event(Timer0Event::Compare, 10);
        assert!(second.expect("run").is_some());
        assert_eq!(session.cpu().cycles(), 8);
    }

    #[test]
    fn timer0_overflow_event_stops_session() {
        let mut session = pin_session(vec![0xCFFF]);
        configure_timer0(&mut session, 200);
        let trace = session.run_until_timer0_event(Timer0Event::Overflow, 200);
        assert!(trace.expect("run").is_some());
        assert_eq!(session.cpu().cycles(), 256);
        assert!(session.timer0_overflow());
    }

    #[test]
    fn timer0_event_queue_records_time_and_cycles() {
        let mut session = pin_session(vec![0xCFFF]);
        configure_timer0(&mut session, 4);
        session.cpu_mut().timer0().set_ctc(true);
        let trace = session.run_until_timer0_event(Timer0Event::Compare, 10);
        assert!(trace.expect("run").is_some());
        assert_eq!(session.event_count(), 1);
        let event = session.take_event().expect("event");
        assert_eq!(event.kind, CoSimEventKind::Timer0Compare);
        assert_eq!(event.cycles, 4);
        assert_eq!(event.time.as_ps(), 10_000_000);
        assert_eq!(session.event_count(), 0);
    }

    #[test]
    fn pin_event_queue_records_level() {
        let mut session = pin_session(blink_program());
        assert!(session.run_until_pin_change(5000).expect("run").is_some());
        let event = session.take_event().expect("event");
        assert_eq!(
            event.kind,
            CoSimEventKind::PinChanged {
                index: 0,
                level: true
            }
        );
        assert!(event.time > kernel::Time::ZERO);
    }

    #[test]
    fn timer_pwm_drives_analog_node() {
        let mut session = pwm_session(10_000.0, 1e-7);
        let (low, high) = run_pwm(&mut session, 800);
        assert!(low < 2.7, "low={low}");
        assert!(high > 2.3, "high={high}");
    }

    fn pwm_session(resistance: f64, capacitance: f64) -> AvrSession {
        let (circuit, source, node) = rc_circuit(resistance, capacitance);
        let pwm = PwmLink {
            source,
            node,
            high: 5.0,
            low: 0.0,
        };
        let mut session = pwm_link_session(&circuit, pwm);
        configure_pwm(&mut session);
        session
    }

    fn pwm_link_session(circuit: &Circuit, pwm: PwmLink) -> AvrSession {
        AvrSession::with_links(
            vec![0xCFFF],
            circuit,
            &config(1e-5, 1000),
            Vec::new(),
            None,
            Some(pwm),
        )
        .expect("session")
    }

    fn configure_pwm(session: &mut AvrSession) {
        configure_timer0(session, 128);
        session.cpu_mut().timer0().set_pwm(false);
    }

    fn configure_timer0(session: &mut AvrSession, compare: u8) {
        session
            .cpu_mut()
            .set_timer0_prescaler(1)
            .expect("prescaler");
        session.cpu_mut().timer0().set_compare(compare);
    }

    fn run_pwm(session: &mut AvrSession, steps: usize) -> (f64, f64) {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for _ in 0..steps {
            session.step().expect("step");
            let voltage = session.pwm_voltage().unwrap_or(0.0);
            low = low.min(voltage);
            high = high.max(voltage);
        }
        (low, high)
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
