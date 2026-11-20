//! Deterministic digital to analog co-simulation with analog feedback.

#![forbid(unsafe_code)]

use std::collections::VecDeque;

use analysis::{TransientConfig, TransientSession};
use avr::{Adc, AvrTrace, Cpu, ExternalPin, PinElectricalConfig};
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

const AVR_PORT_COUNT: u8 = 3;
const AVR_PORT_WIDTH: u8 = 8;

/// Number of IO pins modeled by the AVR.
pub const AVR_IO_PIN_COUNT: usize = AVR_PORT_COUNT as usize * AVR_PORT_WIDTH as usize;

/// Pin-to-node map for all modeled AVR ports.
#[derive(Clone, Copy)]
pub struct AvrPinMap {
    pins: [Option<AvrPin>; AVR_IO_PIN_COUNT],
}

impl Default for AvrPinMap {
    fn default() -> Self {
        Self {
            pins: [None; AVR_IO_PIN_COUNT],
        }
    }
}

impl AvrPinMap {
    /// Builds an empty pin map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one pin and rejects duplicate or invalid locations.
    pub fn add(&mut self, pin: AvrPin) -> Result<(), SimError> {
        let index = pin_index(pin.port, pin.bit)?;
        if self.pins[index].is_some() {
            return Err(SimError::InvalidNetlist(format!(
                "AVR pin port {} bit {} already mapped",
                pin.port, pin.bit
            )));
        }
        self.pins[index] = Some(pin);
        Ok(())
    }

    /// Returns the pin at one port and bit.
    #[must_use]
    pub fn get(&self, port: u8, bit: u8) -> Option<AvrPin> {
        let index = pin_index(port, bit).ok()?;
        self.pins[index]
    }

    /// Removes and returns the pin at one port and bit.
    pub fn remove(&mut self, port: u8, bit: u8) -> Option<AvrPin> {
        let index = pin_index(port, bit).ok()?;
        self.pins[index].take()
    }

    /// Returns the number of mapped pins.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pins.iter().flatten().count()
    }

    /// Returns true when no pin is mapped.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns mapped pins in port and bit order.
    #[must_use]
    pub fn links(&self) -> Vec<AvrPin> {
        self.pins.iter().flatten().copied().collect()
    }
}

fn pin_index(port: u8, bit: u8) -> Result<usize, SimError> {
    if port >= AVR_PORT_COUNT {
        return Err(SimError::InvalidNetlist(format!(
            "AVR port out of range: {port}"
        )));
    }
    if bit >= AVR_PORT_WIDTH {
        return Err(SimError::InvalidNetlist(format!(
            "AVR bit out of range: {bit}"
        )));
    }
    Ok(usize::from(port) * usize::from(AVR_PORT_WIDTH) + usize::from(bit))
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

/// Last-step event state and monotonic counter snapshots.
#[derive(Clone, Copy)]
struct EventState {
    timer0: Timer0EventState,
    adc_conversion_count: u32,
    pwm_level: bool,
    uart_tx_level: bool,
}

impl Default for EventState {
    fn default() -> Self {
        Self {
            timer0: Timer0EventState::default(),
            adc_conversion_count: 0,
            pwm_level: false,
            uart_tx_level: true,
        }
    }
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

/// Analog node observed by a voltage breakpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalogProbe {
    /// Pin link index.
    Pin(usize),
    /// ADC link index.
    Adc(usize),
    /// PWM link index.
    Pwm(usize),
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
    /// A UART byte was received.
    UartRx(u8),
    /// A UART byte with the ninth data bit set was received.
    UartRx9(u8),
    /// An ADC conversion completed.
    AdcConversion {
        /// Selected ADC channel.
        channel: u8,
        /// Converted 10-bit code.
        code: u16,
    },
    /// Timer0 PWM output changed level.
    PwmEdge {
        /// New PWM level.
        high: bool,
    },
    /// UART TX line changed level.
    UartTxEdge {
        /// New TX line level.
        high: bool,
    },
}

/// AVR UART TX line linked to an analog source.
#[derive(Clone, Copy)]
pub struct UartTxLink {
    /// Analog voltage source driven by the TX line.
    pub source: DeviceId,
    /// Analog node observed after each step.
    pub node: NodeId,
    /// Source voltage for a high line.
    pub high: f64,
    /// Source voltage for a low line.
    pub low: f64,
}

/// Analog node sampled as the AVR UART RX line.
#[derive(Clone, Copy)]
pub struct UartRxLink {
    /// Analog node sampled for the RX line.
    pub node: NodeId,
    /// Voltage threshold for a high line.
    pub threshold: f64,
}

struct UartState {
    link: Option<UartLink>,
    tx: Option<UartTxLink>,
    tx_voltage: f64,
    rx: Option<UartRxLink>,
    rx_level: bool,
    rx_voltage: f64,
    rx_count: u32,
}

impl Default for UartState {
    fn default() -> Self {
        Self {
            link: None,
            tx: None,
            tx_voltage: 0.0,
            rx: None,
            rx_level: true,
            rx_voltage: 0.0,
            rx_count: 0,
        }
    }
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
    adcs: Vec<AdcLink>,
    pwms: Vec<PwmLink>,
    uart_state: UartState,
    adc_voltages: Vec<f64>,
    pwm_voltages: Vec<f64>,
    event_state: EventState,
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

    /// Builds a session from a validated pin map.
    pub fn with_pin_map(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        map: &AvrPinMap,
    ) -> Result<Self, SimError> {
        Self::with_pins(program, circuit, config, map.links())
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
        Self::with_bridges(
            program,
            circuit,
            config,
            pins,
            adc.into_iter().collect(),
            pwm,
        )
    }

    /// Builds a session with multiple ADC links and an optional PWM link.
    pub fn with_bridges(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pins: Vec<AvrPin>,
        adcs: Vec<AdcLink>,
        pwm: Option<PwmLink>,
    ) -> Result<Self, SimError> {
        Self::with_pwm_links(
            program,
            circuit,
            config,
            pins,
            adcs,
            pwm.into_iter().collect(),
        )
    }

    /// Builds a session with multiple ADC and PWM links.
    pub fn with_pwm_links(
        program: Vec<u16>,
        circuit: &Circuit,
        config: &TransientConfig,
        pins: Vec<AvrPin>,
        adcs: Vec<AdcLink>,
        pwms: Vec<PwmLink>,
    ) -> Result<Self, SimError> {
        validate_links(&pins, &adcs, &pwms)?;
        let cpu = Cpu::new(program)?;
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self::assemble(cpu, analog, pins, adcs, pwms))
    }

    fn assemble(
        cpu: Cpu,
        analog: TransientSession,
        pins: Vec<AvrPin>,
        adcs: Vec<AdcLink>,
        pwms: Vec<PwmLink>,
    ) -> Self {
        Self {
            cpu,
            analog,
            pins: pin_states(pins),
            adc_voltages: vec![0.0; adcs.len()],
            adcs,
            pwm_voltages: vec![0.0; pwms.len()],
            pwms,
            uart_state: UartState::default(),
            event_state: EventState::default(),
            events: VecDeque::new(),
        }
    }

    /// Attaches an external byte stream to the AVR USART.
    #[must_use]
    pub fn with_uart(mut self, link: UartLink) -> Self {
        self.uart_state.link = Some(link);
        self
    }

    /// Attaches an analog source to the AVR UART TX line.
    pub fn with_uart_tx(mut self, link: UartTxLink) -> Result<Self, SimError> {
        validate_voltage_pair(link.high, link.low)?;
        self.uart_state.tx = Some(link);
        Ok(self)
    }

    /// Attaches an analog node to the AVR UART RX line.
    pub fn with_uart_rx(mut self, link: UartRxLink) -> Result<Self, SimError> {
        if !link.threshold.is_finite() || link.threshold <= 0.0 {
            return Err(SimError::InvalidNetlist(
                "UART RX threshold must be positive".to_string(),
            ));
        }
        self.uart_state.rx = Some(link);
        self.uart_state.rx_level = true;
        Ok(self)
    }

    /// Steps one AVR instruction and one analog step.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        let trace = self.step_cpu()?;
        let driven = self.drive_sources()?;
        self.drive_uart_tx()?;
        self.drive_pwms()?;
        let sample = self.analog.step()?;
        self.update_pins(&driven, &sample);
        self.update_adcs(&sample);
        self.update_pwms(&sample);
        self.update_uart_tx(&sample);
        self.update_uart_rx_line(&sample);
        Ok(trace)
    }

    fn step_cpu(&mut self) -> Result<Option<AvrTrace>, SimError> {
        self.apply_adcs();
        self.apply_uart_rx();
        self.apply_uart_rx_line();
        self.apply_inputs()?;
        let trace = self.cpu.step()?;
        let cycles = trace.as_ref().map_or(1, |trace| trace.cycles).max(1);
        self.update_timer0_events();
        self.record_timer0_events(cycles);
        self.record_pwm_event(cycles);
        self.drain_uart_tx();
        self.record_adc_conversions();
        self.record_uart_rx();
        self.record_uart_tx_edge(cycles);
        Ok(trace)
    }

    fn apply_uart_rx(&mut self) {
        if self.uart_state.rx.is_some() {
            return;
        }
        let Some(value) = self.pending_uart_rx() else {
            return;
        };
        if self.cpu.uart().rx_ready() {
            return;
        }
        self.cpu.uart().receive(value);
        if let Some(link) = self.uart_state.link.as_mut() {
            link.rx.pop_front();
        }
    }

    fn pending_uart_rx(&self) -> Option<u8> {
        self.uart_state
            .link
            .as_ref()
            .and_then(|link| link.rx.front().copied())
    }

    fn apply_uart_rx_line(&mut self) {
        if self.uart_state.rx.is_none() {
            return;
        }
        self.cpu.uart().set_rx_line(self.uart_state.rx_level);
    }

    fn drain_uart_tx(&mut self) {
        if self.uart_state.link.is_none() {
            return;
        }
        while let Some(value) = self.cpu.uart().take_tx_byte() {
            if let Some(link) = self.uart_state.link.as_mut() {
                link.tx.push_back(value);
            }
            self.push_event(CoSimEventKind::UartTx(value));
        }
    }

    fn record_timer0_events(&mut self, cycles: u8) {
        if self.event_state.timer0.overflow {
            let offset = self.cpu.timer0().overflow_cycle().unwrap_or(cycles.into());
            let time = self.event_time_at(offset, cycles);
            self.push_event_at(CoSimEventKind::Timer0Overflow, time);
        }
        if self.event_state.timer0.compare {
            let offset = self.cpu.timer0().compare_cycle().unwrap_or(cycles.into());
            let time = self.event_time_at(offset, cycles);
            self.push_event_at(CoSimEventKind::Timer0Compare, time);
        }
    }

    fn event_time_at(&self, offset: u32, cycles: u8) -> Time {
        let cycles = u32::from(cycles.max(1));
        let fraction = f64::from(offset.min(cycles)) / f64::from(cycles);
        let step = self.analog.step_time();
        let delta = (step.as_ps() as f64 * fraction).round() as u64;
        self.analog.time().saturating_add(Time::from_ps(delta))
    }

    fn record_adc_conversions(&mut self) {
        let (count, channel, code) = {
            let adc = self.cpu.adc();
            let code = u16::from(adc.read_data_low()) | (u16::from(adc.read_data_high()) << 8);
            (adc.conversion_count(), adc.channel(), code)
        };
        if count.wrapping_sub(self.event_state.adc_conversion_count) != 0 {
            self.push_event(CoSimEventKind::AdcConversion { channel, code });
        }
        self.event_state.adc_conversion_count = count;
    }

    fn record_uart_rx(&mut self) {
        let count = self.cpu.uart().rx_count();
        if count.wrapping_sub(self.uart_state.rx_count) != 0 {
            let value = self.cpu.uart().rx_byte();
            if self.cpu.uart().rx_ninth() {
                self.push_event(CoSimEventKind::UartRx9(value));
            } else {
                self.push_event(CoSimEventKind::UartRx(value));
            }
        }
        self.uart_state.rx_count = count;
    }

    fn record_pwm_event(&mut self, cycles: u8) {
        let high = self.cpu.pwm_a();
        if high == self.event_state.pwm_level {
            return;
        }
        let offset = self.cpu.timer0().pwm_cycle().unwrap_or(cycles.into());
        let time = self.event_time_at(offset, cycles);
        self.push_event_at(CoSimEventKind::PwmEdge { high }, time);
        self.event_state.pwm_level = high;
    }

    fn record_uart_tx_edge(&mut self, cycles: u8) {
        let edge = self.cpu.uart().take_tx_edge_cycle();
        if self.uart_state.tx.is_none() {
            return;
        }
        let high = self.cpu.uart().tx_line();
        if high == self.event_state.uart_tx_level {
            return;
        }
        let offset = edge.unwrap_or(cycles.into());
        let time = self.event_time_at(offset, cycles);
        self.push_event_at(CoSimEventKind::UartTxEdge { high }, time);
        self.event_state.uart_tx_level = high;
    }

    fn push_event(&mut self, kind: CoSimEventKind) {
        let time = self.analog.time().saturating_add(self.analog.step_time());
        self.push_event_at(kind, time);
    }

    fn push_event_at(&mut self, kind: CoSimEventKind, time: Time) {
        self.events.push_back(CoSimEvent {
            cycles: self.cpu.cycles(),
            time,
            kind,
        });
    }

    fn update_timer0_events(&mut self) {
        let (overflow, compare) = {
            let timer = self.cpu.timer0();
            (timer.overflow_count(), timer.compare_count())
        };
        let events = &mut self.event_state.timer0;
        events.overflow = overflow.wrapping_sub(events.overflow_count) != 0;
        events.compare = compare.wrapping_sub(events.compare_count) != 0;
        events.overflow_count = overflow;
        events.compare_count = compare;
    }

    /// Returns true when Timer0 overflowed on the last step.
    #[must_use]
    pub fn timer0_overflow(&self) -> bool {
        self.event_state.timer0.overflow
    }

    /// Returns true when Timer0 matched its compare value on the last step.
    #[must_use]
    pub fn timer0_compare(&self) -> bool {
        self.event_state.timer0.compare
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

    /// Removes and returns all queued co-simulation events.
    pub fn drain_events(&mut self) -> Vec<CoSimEvent> {
        self.events.drain(..).collect()
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
            Timer0Event::Overflow => self.event_state.timer0.overflow,
            Timer0Event::Compare => self.event_state.timer0.compare,
        }
    }

    fn apply_adcs(&mut self) {
        for (link, voltage) in self.adcs.iter().zip(&self.adc_voltages) {
            let code = adc_code(*voltage, link.reference);
            self.cpu.adc().set_input(link.channel, code);
        }
    }

    fn drive_pwms(&mut self) -> Result<(), SimError> {
        let high = self.cpu.pwm_a();
        for link in &self.pwms {
            let value = if high { link.high } else { link.low };
            self.analog.set_source_voltage(link.source, value)?;
        }
        Ok(())
    }

    fn drive_uart_tx(&mut self) -> Result<(), SimError> {
        let Some(link) = self.uart_state.tx else {
            return Ok(());
        };
        let value = if self.cpu.uart().tx_line() {
            link.high
        } else {
            link.low
        };
        self.analog.set_source_voltage(link.source, value)
    }

    fn update_adcs(&mut self, sample: &analysis::TransientSample) {
        for (index, link) in self.adcs.iter().enumerate() {
            self.adc_voltages[index] = sample.solution.voltage(link.node).unwrap_or(0.0);
        }
    }

    fn update_pwms(&mut self, sample: &analysis::TransientSample) {
        for (index, link) in self.pwms.iter().enumerate() {
            self.pwm_voltages[index] = sample.solution.voltage(link.node).unwrap_or(0.0);
        }
    }

    fn update_uart_tx(&mut self, sample: &analysis::TransientSample) {
        let Some(link) = self.uart_state.tx else {
            return;
        };
        self.uart_state.tx_voltage = sample.solution.voltage(link.node).unwrap_or(0.0);
    }

    fn update_uart_rx_line(&mut self, sample: &analysis::TransientSample) {
        let Some(link) = self.uart_state.rx else {
            return;
        };
        let voltage = sample.solution.voltage(link.node).unwrap_or(0.0);
        self.uart_state.rx_voltage = voltage;
        self.uart_state.rx_level = voltage >= link.threshold;
    }

    /// Returns the first ADC node voltage.
    #[must_use]
    pub fn adc_voltage(&self) -> Option<f64> {
        self.adc_voltage_at(0)
    }

    /// Returns one ADC node voltage by link index.
    #[must_use]
    pub fn adc_voltage_at(&self, index: usize) -> Option<f64> {
        self.adc_voltages.get(index).copied()
    }

    /// Returns the first PWM node voltage.
    #[must_use]
    pub fn pwm_voltage(&self) -> Option<f64> {
        self.pwm_voltage_at(0)
    }

    /// Returns one PWM node voltage by link index.
    #[must_use]
    pub fn pwm_voltage_at(&self, index: usize) -> Option<f64> {
        self.pwm_voltages.get(index).copied()
    }

    /// Returns the attached UART link.
    #[must_use]
    pub fn uart_link(&self) -> Option<&UartLink> {
        self.uart_state.link.as_ref()
    }

    /// Returns the attached UART link for external byte access.
    pub fn uart_link_mut(&mut self) -> Option<&mut UartLink> {
        self.uart_state.link.as_mut()
    }

    /// Returns the last UART TX analog node voltage.
    #[must_use]
    pub fn uart_tx_voltage(&self) -> Option<f64> {
        self.uart_state.tx.map(|_| self.uart_state.tx_voltage)
    }

    /// Returns the last UART RX analog node voltage.
    #[must_use]
    pub fn uart_rx_voltage(&self) -> Option<f64> {
        self.uart_state.rx.map(|_| self.uart_state.rx_voltage)
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

    /// Returns the number of mapped pins in the session.
    #[must_use]
    pub fn pin_count(&self) -> usize {
        self.pins.len()
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

    /// Runs until one pin node reaches the threshold or the limit is reached.
    pub fn run_until_pin_voltage(
        &mut self,
        index: usize,
        threshold: f64,
        max_steps: usize,
    ) -> Result<Option<AvrTrace>, SimError> {
        self.run_until_voltage(AnalogProbe::Pin(index), threshold, max_steps)
    }

    /// Runs until one analog probe reaches the threshold or the limit is reached.
    pub fn run_until_voltage(
        &mut self,
        probe: AnalogProbe,
        threshold: f64,
        max_steps: usize,
    ) -> Result<Option<AvrTrace>, SimError> {
        self.validate_probe(probe)?;
        validate_threshold(threshold)?;
        for _ in 0..max_steps {
            let trace = self.step()?;
            if self.probe_voltage(probe) >= threshold {
                return Ok(trace);
            }
        }
        Ok(None)
    }

    fn validate_probe(&self, probe: AnalogProbe) -> Result<(), SimError> {
        let exists = match probe {
            AnalogProbe::Pin(index) => index < self.pins.len(),
            AnalogProbe::Adc(index) => index < self.adc_voltages.len(),
            AnalogProbe::Pwm(index) => index < self.pwm_voltages.len(),
        };
        if exists {
            return Ok(());
        }
        Err(SimError::InvalidNetlist(
            "analog probe out of range".to_string(),
        ))
    }

    fn probe_voltage(&self, probe: AnalogProbe) -> f64 {
        match probe {
            AnalogProbe::Pin(index) => self
                .pins
                .get(index)
                .map_or(f64::NEG_INFINITY, |pin| pin.voltage),
            AnalogProbe::Adc(index) => self.adc_voltage_at(index).unwrap_or(f64::NEG_INFINITY),
            AnalogProbe::Pwm(index) => self.pwm_voltage_at(index).unwrap_or(f64::NEG_INFINITY),
        }
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

fn validate_threshold(threshold: f64) -> Result<(), SimError> {
    if !threshold.is_finite() {
        return Err(SimError::InvalidNetlist(
            "voltage threshold must be finite".to_string(),
        ));
    }
    Ok(())
}

fn adc_code(voltage: f64, reference: f64) -> u16 {
    if reference <= 0.0 {
        return 0;
    }
    let ratio = (voltage / reference).clamp(0.0, 1.0);
    (ratio * 1023.0).round() as u16
}

fn validate_adcs(adcs: &[AdcLink]) -> Result<(), SimError> {
    for (index, link) in adcs.iter().enumerate() {
        validate_adc_link(link)?;
        if adcs[..index]
            .iter()
            .any(|other| other.channel == link.channel)
        {
            return Err(SimError::InvalidNetlist(format!(
                "ADC channel already mapped: {}",
                link.channel
            )));
        }
    }
    Ok(())
}

fn validate_adc_link(link: &AdcLink) -> Result<(), SimError> {
    if usize::from(link.channel) >= Adc::CHANNELS {
        return Err(SimError::InvalidNetlist(format!(
            "ADC channel out of range: {}",
            link.channel
        )));
    }
    if !link.reference.is_finite() || link.reference <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "ADC reference must be positive".to_string(),
        ));
    }
    Ok(())
}

fn validate_pwms(pwms: &[PwmLink]) -> Result<(), SimError> {
    for (index, link) in pwms.iter().enumerate() {
        validate_pwm_link(link)?;
        if pwms[..index]
            .iter()
            .any(|other| other.source == link.source)
        {
            return Err(SimError::InvalidNetlist(
                "PWM source already mapped".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_pwm_link(link: &PwmLink) -> Result<(), SimError> {
    validate_voltage_pair(link.high, link.low)
}

fn validate_voltage_pair(high: f64, low: f64) -> Result<(), SimError> {
    if !high.is_finite() || !low.is_finite() {
        return Err(SimError::InvalidNetlist(
            "voltage must be finite".to_string(),
        ));
    }
    Ok(())
}

fn validate_pins(pins: &[AvrPin]) -> Result<(), SimError> {
    for pin in pins {
        pin_index(pin.port, pin.bit)?;
    }
    Ok(())
}

fn validate_links(pins: &[AvrPin], adcs: &[AdcLink], pwms: &[PwmLink]) -> Result<(), SimError> {
    validate_pins(pins)?;
    validate_adcs(adcs)?;
    validate_pwms(pwms)?;
    if pins.is_empty() && adcs.is_empty() && pwms.is_empty() {
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
        AVR_IO_PIN_COUNT, AdcLink, AnalogProbe, AvrPin, AvrPinMap, AvrSession, CoSimEventKind,
        MixedSession, PinInput, PinLink, PwmLink, Timer0Event, UartLink, UartRxLink, UartTxLink,
    };
    use analysis::TransientConfig;
    use avr::PinElectricalConfig;
    use cells::DigitalBuilder;
    use cells::catalog_digital::{DigitalEntry, digital_entry};
    use kernel::{Circuit, CircuitBuilder, DeviceId, DeviceKind, NodeId, SimError};
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

    fn dual_fixed_circuit(first: f64, second: f64) -> (Circuit, NodeId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let node_a = builder.node("a").expect("node");
        let node_b = builder.node("b").expect("node");
        let ground = builder.ground();
        add_source(&mut builder, "V1", node_a, ground, first);
        add_source(&mut builder, "V2", node_b, ground, second);
        (builder.build().expect("circuit"), node_a, node_b)
    }

    fn multi_pin_circuit(count: usize) -> (Circuit, Vec<(DeviceId, NodeId)>) {
        let mut builder = CircuitBuilder::new();
        let ground = builder.ground();
        let mut links = Vec::with_capacity(count);
        for index in 0..count {
            let node = builder.node(format!("p{index}")).expect("node");
            let source = add_source(&mut builder, &format!("V{index}"), node, ground, 0.0);
            links.push((source, node));
        }
        (builder.build().expect("circuit"), links)
    }

    fn test_pin(port: u8, bit: u8, link: (DeviceId, NodeId)) -> AvrPin {
        AvrPin {
            port,
            bit,
            source: link.0,
            node: link.1,
            config: PinElectricalConfig::default(),
            external_resistance: 1000.0,
        }
    }

    fn full_pin_map(links: &[(DeviceId, NodeId)]) -> AvrPinMap {
        let mut map = AvrPinMap::new();
        for (index, link) in links.iter().enumerate() {
            let port = (index / 8) as u8;
            let bit = (index % 8) as u8;
            map.add(test_pin(port, bit, *link)).expect("pin");
        }
        map
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
    fn run_until_pin_voltage_stops_at_threshold() {
        let (circuit, source, node) = thevenin_rc_circuit(45.0, 1000.0, 1e-6);
        let pin = test_pin(0, 5, (source, node));
        let mut session = AvrSession::new(blink_program(), &circuit, &config(1e-5, 10_000), pin)
            .expect("session");
        let trace = session.run_until_pin_voltage(0, 3.0, 5000);
        assert!(trace.expect("run").is_some());
        assert!(session.node_voltage(0).unwrap_or(0.0) >= 3.0);
        assert!(session.run_until_pin_voltage(1, 3.0, 1).is_err());
        assert!(session.run_until_pin_voltage(0, f64::NAN, 1).is_err());
    }

    #[test]
    fn run_until_voltage_supports_adc_and_pwm_nodes() {
        let (circuit, node) = fixed_circuit(2.5);
        let mut adc = adc_bridge_session(circuit, vec![adc_link(0, node)]);
        let adc_trace = adc.run_until_voltage(AnalogProbe::Adc(0), 2.4, 10);
        assert!(adc_trace.expect("adc run").is_some());
        let mut pwm = pwm_session(10_000.0, 1e-7);
        let pwm_trace = pwm.run_until_voltage(AnalogProbe::Pwm(0), 2.5, 800);
        assert!(pwm_trace.expect("pwm run").is_some());
        assert!(pwm.run_until_voltage(AnalogProbe::Adc(0), 2.5, 1).is_err());
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
        let adc = adc_link(0, node);
        let mut session = adc_session(circuit, adc);
        session.step().expect("settle");
        assert!((session.adc_voltage().unwrap_or(0.0) - 2.5).abs() < 0.01);
        let code = convert_adc(&mut session, 0);
        assert!((500..=520).contains(&code), "code={code}");
        let event = session.take_event().expect("adc event");
        assert_eq!(
            event.kind,
            CoSimEventKind::AdcConversion { channel: 0, code }
        );
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
    fn multiple_adc_channels_sample_independent_nodes() {
        let (circuit, node_a, node_b) = dual_fixed_circuit(1.25, 3.75);
        let adcs = vec![adc_link(0, node_a), adc_link(1, node_b)];
        let mut session = adc_bridge_session(circuit, adcs);
        session.step().expect("settle");
        assert!((session.adc_voltage_at(0).unwrap_or(0.0) - 1.25).abs() < 0.01);
        assert!((session.adc_voltage_at(1).unwrap_or(0.0) - 3.75).abs() < 0.01);
        let low = i32::from(convert_adc(&mut session, 0));
        let high = i32::from(convert_adc(&mut session, 1));
        assert!((low - 256).abs() <= 5, "low={low}");
        assert!((high - 767).abs() <= 5, "high={high}");
    }

    fn adc_link(channel: u8, node: NodeId) -> AdcLink {
        AdcLink {
            channel,
            node,
            reference: 5.0,
        }
    }

    fn adc_bridge_session(circuit: Circuit, adcs: Vec<AdcLink>) -> AvrSession {
        AvrSession::with_bridges(
            vec![0xCFFF],
            &circuit,
            &config(1e-5, 200),
            Vec::new(),
            adcs,
            None,
        )
        .expect("session")
    }

    #[test]
    fn adc_links_reject_duplicate_and_invalid() {
        let (circuit, node_a, node_b) = dual_fixed_circuit(1.25, 3.75);
        let duplicate = vec![adc_link(0, node_a), adc_link(0, node_b)];
        assert!(try_adc_session(&circuit, duplicate).is_err());
        let invalid = vec![AdcLink {
            channel: 16,
            node: node_a,
            reference: 5.0,
        }];
        assert!(try_adc_session(&circuit, invalid).is_err());
        let zero = vec![AdcLink {
            channel: 0,
            node: node_a,
            reference: 0.0,
        }];
        assert!(try_adc_session(&circuit, zero).is_err());
    }

    fn try_adc_session(circuit: &Circuit, adcs: Vec<AdcLink>) -> Result<AvrSession, SimError> {
        AvrSession::with_pwm_links(
            vec![0xCFFF],
            circuit,
            &config(1e-5, 10),
            Vec::new(),
            adcs,
            Vec::new(),
        )
    }

    #[test]
    fn pwm_links_reject_duplicate_sources_and_invalid_voltage() {
        let (circuit, links) = dual_rc_circuit();
        let mut duplicate = pwm_links(&links);
        duplicate[1].source = duplicate[0].source;
        assert!(try_pwm_session(&circuit, duplicate).is_err());
        let mut invalid = pwm_links(&links);
        invalid[0].high = f64::INFINITY;
        assert!(try_pwm_session(&circuit, invalid).is_err());
    }

    #[test]
    fn pins_reject_invalid_locations() {
        let (circuit, links) = multi_pin_circuit(1);
        let mut invalid = test_pin(0, 5, links[0]);
        invalid.port = 3;
        assert!(try_pin_session(&circuit, vec![invalid]).is_err());
        invalid.port = 0;
        invalid.bit = 8;
        assert!(try_pin_session(&circuit, vec![invalid]).is_err());
    }

    fn try_pin_session(circuit: &Circuit, pins: Vec<AvrPin>) -> Result<AvrSession, SimError> {
        AvrSession::with_pins(vec![0xCFFF], circuit, &config(1e-5, 10), pins)
    }

    fn try_pwm_session(circuit: &Circuit, pwms: Vec<PwmLink>) -> Result<AvrSession, SimError> {
        AvrSession::with_pwm_links(
            vec![0xCFFF],
            circuit,
            &config(1e-5, 10),
            Vec::new(),
            Vec::new(),
            pwms,
        )
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

    fn uart_tx_circuit() -> (Circuit, DeviceId, NodeId, DeviceId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let tx_node = builder.node("uart").expect("node");
        let pin_node = builder.node("pin").expect("node");
        let ground = builder.ground();
        let tx_source = add_source(&mut builder, "V1", tx_node, ground, 0.0);
        let pin_source = add_source(&mut builder, "V2", pin_node, ground, 0.0);
        (
            builder.build().expect("circuit"),
            tx_source,
            tx_node,
            pin_source,
            pin_node,
        )
    }

    #[test]
    fn uart_tx_line_drives_analog_node() {
        let mut session = uart_tx_session();
        session.cpu_mut().uart().write_control(1, 0x08);
        session.cpu_mut().uart().write_tx(0x55);
        let (low, high) = run_uart_tx(&mut session, 100);
        assert!(low < 1.0, "low={low}");
        assert!(high > 4.0, "high={high}");
        let edges = take_uart_tx_edges(&mut session);
        assert!(edges.contains(&true));
        assert!(edges.contains(&false));
    }

    fn uart_tx_session() -> AvrSession {
        let (circuit, tx_source, tx_node, pin_source, pin_node) = uart_tx_circuit();
        let pin = test_pin(0, 5, (pin_source, pin_node));
        AvrSession::new(vec![0xCFFF], &circuit, &config(1e-5, 1000), pin)
            .expect("session")
            .with_uart_tx(UartTxLink {
                source: tx_source,
                node: tx_node,
                high: 5.0,
                low: 0.0,
            })
            .expect("uart tx")
    }

    fn take_uart_tx_edges(session: &mut AvrSession) -> Vec<bool> {
        session
            .drain_events()
            .into_iter()
            .filter_map(|event| match event.kind {
                CoSimEventKind::UartTxEdge { high } => Some(high),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn uart_tx_line_loops_back_to_rx() {
        let mut session = uart_loop_session();
        session.cpu_mut().uart().write_control(1, 0x18);
        session.cpu_mut().uart().write_tx(0x5A);
        assert_eq!(receive_uart(&mut session, 400), Some(0x5A));
        assert_eq!(take_uart_rx(&mut session), Some(0x5A));
    }

    #[test]
    fn uart_nine_bit_loopback_marks_ninth() {
        let mut session = uart_loop_session();
        session.cpu_mut().uart().write_control(1, 0x1D);
        session.cpu_mut().uart().write_tx(0x5A);
        assert_eq!(receive_uart(&mut session, 400), Some(0x5A));
        assert!(session.cpu_mut().uart().rx_ninth());
        let events = session.drain_events();
        assert!(
            events
                .iter()
                .any(|event| event.kind == CoSimEventKind::UartRx9(0x5A))
        );
    }

    fn take_uart_rx(session: &mut AvrSession) -> Option<u8> {
        while let Some(event) = session.take_event() {
            if let CoSimEventKind::UartRx(value) = event.kind {
                return Some(value);
            }
        }
        None
    }

    fn uart_loop_session() -> AvrSession {
        let (circuit, tx_source, tx_node, pin_source, pin_node) = uart_tx_circuit();
        let pin = test_pin(0, 5, (pin_source, pin_node));
        AvrSession::new(vec![0xCFFF], &circuit, &config(1e-5, 1000), pin)
            .expect("session")
            .with_uart_tx(UartTxLink {
                source: tx_source,
                node: tx_node,
                high: 5.0,
                low: 0.0,
            })
            .expect("uart tx")
            .with_uart_rx(UartRxLink {
                node: tx_node,
                threshold: 2.5,
            })
            .expect("uart rx")
    }

    fn receive_uart(session: &mut AvrSession, steps: usize) -> Option<u8> {
        for _ in 0..steps {
            session.step().expect("step");
            if session.cpu_mut().uart().rx_ready() {
                return Some(session.cpu_mut().uart().read_rx());
            }
        }
        None
    }

    fn run_uart_tx(session: &mut AvrSession, steps: usize) -> (f64, f64) {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for _ in 0..steps {
            session.step().expect("step");
            let voltage = session.uart_tx_voltage().unwrap_or(0.0);
            low = low.min(voltage);
            high = high.max(voltage);
        }
        (low, high)
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
        assert_eq!(event.time.as_ps(), 20_000_000);
        assert_eq!(session.event_count(), 0);
    }

    #[test]
    fn timer0_event_interpolates_within_step() {
        let mut session = pin_session(vec![0xCFFF]);
        configure_timer0(&mut session, 1);
        session.cpu_mut().timer0().set_ctc(true);
        let trace = session.run_until_timer0_event(Timer0Event::Compare, 10);
        assert!(trace.expect("run").is_some());
        let event = session.take_event().expect("event");
        assert_eq!(event.kind, CoSimEventKind::Timer0Compare);
        assert_eq!(event.cycles, 2);
        assert_eq!(event.time.as_ps(), 5_000_000);
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
    fn pin_map_tracks_one_pin_per_location() {
        let (_, links) = multi_pin_circuit(1);
        let pin = test_pin(0, 5, links[0]);
        let mut map = AvrPinMap::new();
        assert!(map.is_empty());
        map.add(pin).expect("add");
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(0, 5).map(|value| value.bit), Some(5));
        assert!(map.add(pin).is_err());
        let mut invalid = pin;
        invalid.port = 3;
        assert!(map.add(invalid).is_err());
        invalid.port = 0;
        invalid.bit = 8;
        assert!(map.add(invalid).is_err());
        assert!(map.get(3, 0).is_none());
        assert!(map.remove(0, 5).is_some());
        assert!(map.is_empty());
    }

    #[test]
    fn avr_pin_map_links_all_io() {
        let (circuit, links) = multi_pin_circuit(AVR_IO_PIN_COUNT);
        let map = full_pin_map(&links);
        assert_eq!(map.len(), AVR_IO_PIN_COUNT);
        let pins = map.links();
        assert_eq!(pins.first().map(|pin| (pin.port, pin.bit)), Some((0, 0)));
        assert_eq!(pins.last().map(|pin| (pin.port, pin.bit)), Some((2, 7)));
        let mut session =
            AvrSession::with_pin_map(blink_program(), &circuit, &config(1e-5, 1000), &map)
                .expect("session");
        session.step().expect("step");
        assert_eq!(session.pin_count(), AVR_IO_PIN_COUNT);
    }

    fn all_pins_program(high: bool) -> Vec<u16> {
        let mut program = vec![0xEF0F, 0xB904, 0xB907, 0xB90A];
        if high {
            program.extend([0xB905, 0xB908, 0xB90B]);
        }
        program.push(0xCFFF);
        program
    }

    fn all_pins_session(circuit: &Circuit, map: &AvrPinMap, high: bool) -> AvrSession {
        AvrSession::with_pin_map(all_pins_program(high), circuit, &config(1e-5, 20), map)
            .expect("session")
    }

    fn assert_all_pin_voltages(high: &AvrSession, low: &AvrSession) {
        for index in 0..AVR_IO_PIN_COUNT {
            let high_voltage = high.node_voltage(index).expect("high node");
            let low_voltage = low.node_voltage(index).expect("low node");
            assert!(high_voltage > 4.0, "pin {index} high={high_voltage}");
            assert!(low_voltage < 1.0, "pin {index} low={low_voltage}");
        }
    }

    #[test]
    fn all_mapped_pins_drive_thevenin_nodes() {
        let (circuit, links) = multi_pin_circuit(AVR_IO_PIN_COUNT);
        let map = full_pin_map(&links);
        let mut high = all_pins_session(&circuit, &map, true);
        let mut low = all_pins_session(&circuit, &map, false);
        for _ in 0..8 {
            high.step().expect("step");
        }
        for _ in 0..6 {
            low.step().expect("step");
        }
        assert_all_pin_voltages(&high, &low);
    }

    #[test]
    fn timer_pwm_drives_analog_node() {
        let mut session = pwm_session(10_000.0, 1e-7);
        let (low, high) = run_pwm(&mut session, 800);
        assert!(low < 2.7, "low={low}");
        assert!(high > 2.3, "high={high}");
        let edges = take_pwm_edges(&mut session);
        assert!(edges.contains(&true));
        assert!(edges.contains(&false));
    }

    fn take_pwm_edges(session: &mut AvrSession) -> Vec<bool> {
        session
            .drain_events()
            .into_iter()
            .filter_map(|event| match event.kind {
                CoSimEventKind::PwmEdge { high } => Some(high),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn timer_pwm_drives_multiple_analog_nodes() {
        let (circuit, links) = dual_rc_circuit();
        let mut session = AvrSession::with_pwm_links(
            vec![0xCFFF],
            &circuit,
            &config(1e-5, 1000),
            Vec::new(),
            Vec::new(),
            pwm_links(&links),
        )
        .expect("session");
        configure_pwm(&mut session);
        let (low, high) = run_pwm_links(&mut session, 800);
        assert!(low < 2.7, "low={low}");
        assert!(high > 2.3, "high={high}");
    }

    fn pwm_links(links: &[(DeviceId, NodeId)]) -> Vec<PwmLink> {
        links
            .iter()
            .map(|(source, node)| PwmLink {
                source: *source,
                node: *node,
                high: 5.0,
                low: 0.0,
            })
            .collect()
    }

    fn run_pwm_links(session: &mut AvrSession, steps: usize) -> (f64, f64) {
        let mut low = f64::INFINITY;
        let mut high = f64::NEG_INFINITY;
        for _ in 0..steps {
            session.step().expect("step");
            for index in 0..2 {
                let voltage = session.pwm_voltage_at(index).unwrap_or(0.0);
                low = low.min(voltage);
                high = high.max(voltage);
            }
        }
        (low, high)
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
