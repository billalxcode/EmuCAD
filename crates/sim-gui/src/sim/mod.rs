//! Background AVR simulation session and worker protocol.

use sim_avr::{Cpu, ExternalPin, PinElectricalConfig};
use sim_board::{Board, HexImage};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CHUNK: usize = 100;
const PACE_CYCLES: u64 = 80_000;
const POLL_MS: u64 = 2;
const FLASH_BYTES: usize = 0x8000;
const WAVE_LIMIT: usize = 2048;
/// Default probe sampling interval in CPU cycles.
pub const DEFAULT_SAMPLE_EVERY: u64 = 100;

/// One PB5/LED sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub cycle: u64,
    pub pb5_high: bool,
    pub pb5_millivolts: i32,
    pub led_on: bool,
    pub led_microamps: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SampleState {
    pb5_high: bool,
    pb5_millivolts: i32,
    led_on: bool,
    led_microamps: i32,
}

impl SampleState {
    fn of(machine: &SimMachine) -> Self {
        Self {
            pb5_high: machine.pb5_high,
            pb5_millivolts: machine.pb5_millivolts,
            led_on: machine.led_on,
            led_microamps: machine.led_microamps,
        }
    }

    const fn sample(self, cycle: u64) -> Sample {
        Sample {
            cycle,
            pb5_high: self.pb5_high,
            pb5_millivolts: self.pb5_millivolts,
            led_on: self.led_on,
            led_microamps: self.led_microamps,
        }
    }
}

struct WorkerState {
    machine: Option<SimMachine>,
    running: bool,
    last: SampleState,
    sample_every: u64,
    pacer: pacing::Pacer,
    next_pace: u64,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            machine: None,
            running: false,
            last: SampleState::default(),
            sample_every: DEFAULT_SAMPLE_EVERY,
            pacer: pacing::Pacer::new(),
            next_pace: 0,
        }
    }

    fn load(&mut self, words: Vec<u16>, events: &Sender<Event>) {
        load_machine(
            words,
            &mut self.machine,
            &mut self.running,
            &mut self.last,
            events,
        );
        self.restart_pacing();
    }

    fn run(&mut self, value: bool, events: &Sender<Event>) {
        if value {
            self.restart_pacing();
        }
        update_running(&self.machine, &mut self.running, value, events);
    }

    fn restart_pacing(&mut self) {
        let cycles = self.machine_cycles();
        self.pacer.restart(cycles);
        self.next_pace = cycles.saturating_add(PACE_CYCLES);
    }

    fn machine_cycles(&self) -> u64 {
        self.machine.as_ref().map_or(0, SimMachine::cycles)
    }

    fn pace(&mut self) {
        let cycles = self.machine_cycles();
        if cycles < self.next_pace {
            return;
        }
        self.pacer.wait(cycles);
        self.next_pace = cycles.saturating_add(PACE_CYCLES);
    }

    fn step(&mut self, events: &Sender<Event>) {
        step_once(&mut self.machine, &mut self.running, &mut self.last, events);
    }

    fn reset(&mut self, events: &Sender<Event>) {
        reset_machine(&mut self.machine, &mut self.running, &mut self.last, events);
        self.restart_pacing();
    }

    fn queue_uart(&mut self, data: Vec<u8>) {
        queue_uart(&mut self.machine, data);
    }
}

/// Current simulation status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub cycles: u64,
    pub pb5_high: bool,
    pub pb5_millivolts: i32,
    pub led_on: bool,
    pub led_microamps: i32,
    pub running: bool,
    pub loaded: bool,
}

/// Bounded transition history for the PB5 waveform.
#[derive(Default)]
pub struct Waveform {
    samples: VecDeque<Sample>,
}

impl Waveform {
    /// Removes all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Appends one sample, dropping the oldest beyond the limit.
    pub fn push(&mut self, sample: Sample) {
        if self.samples.len() == WAVE_LIMIT {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Returns the sampled transitions.
    #[must_use]
    pub fn samples(&self) -> &VecDeque<Sample> {
        &self.samples
    }

    /// Returns the visible cycle range, extended to the current cycle.
    #[must_use]
    pub fn span(&self, current: u64) -> (u64, u64) {
        let start = self.samples.front().map_or(0, |sample| sample.cycle);
        let last = self.samples.back().map_or(0, |sample| sample.cycle);
        let end = last.max(current);
        if end <= start {
            return (start, start + 1);
        }
        (start, end)
    }

    /// Returns PB5 and LED states at a cycle.
    #[must_use]
    pub fn state_at(&self, cycle: u64) -> (bool, bool) {
        let mut state = (false, false);
        for sample in &self.samples {
            if sample.cycle > cycle {
                break;
            }
            state = (sample.pb5_high, sample.led_on);
        }
        state
    }
}

/// Worker command.
pub enum Command {
    Load(Vec<u16>),
    Run,
    Pause,
    Step,
    Reset,
    Uart(Vec<u8>),
    SampleEvery(u64),
    Shutdown,
}

/// Worker event.
pub enum Event {
    Status(Status),
    Sample(Sample),
    Uart(Vec<u8>),
    UartCleared,
    Error(String),
}

struct SimMachine {
    cpu: Cpu,
    board: Board,
    config: PinElectricalConfig,
    led_on: bool,
    pb5_high: bool,
    pb5_millivolts: i32,
    led_microamps: i32,
    uart_bytes: Vec<u8>,
    uart_rx: VecDeque<u8>,
}

impl SimMachine {
    fn new(words: Vec<u16>) -> Result<Self, String> {
        let cpu = Cpu::new(words).map_err(|error| error.to_string())?;
        Ok(Self {
            cpu,
            board: Board::new(),
            config: PinElectricalConfig::default(),
            led_on: false,
            pb5_high: false,
            pb5_millivolts: 0,
            led_microamps: 0,
            uart_bytes: Vec::new(),
            uart_rx: VecDeque::new(),
        })
    }

    fn step(&mut self) -> Result<bool, String> {
        self.pump_uart_rx();
        let trace = self.cpu.step().map_err(|error| error.to_string())?;
        if trace.is_none() {
            return Ok(false);
        }
        self.update_board()?;
        self.update_uart();
        Ok(true)
    }

    fn update_board(&mut self) -> Result<(), String> {
        let state = self
            .cpu
            .pin_electrical(0, 5, ExternalPin::HighZ, &self.config)
            .map_err(|error| error.to_string())?;
        self.board.set_pb5_voltage(state.voltage);
        self.led_on = self.board.led().is_on();
        self.pb5_high = state.voltage >= self.config.input_threshold;
        self.pb5_millivolts = (state.voltage * 1000.0).round() as i32;
        self.led_microamps = (self.board.led().current() * 1_000_000.0).round() as i32;
        Ok(())
    }

    fn queue_uart(&mut self, data: &[u8]) {
        let space = 4096usize.saturating_sub(self.uart_rx.len());
        self.uart_rx.extend(data.iter().take(space).copied());
    }

    fn pump_uart_rx(&mut self) {
        if self.cpu.uart().rx_ready() {
            return;
        }
        let Some(byte) = self.uart_rx.pop_front() else {
            return;
        };
        self.cpu.uart().receive(byte);
    }

    fn update_uart(&mut self) {
        while let Some(byte) = self.cpu.uart().take_tx_byte() {
            self.uart_bytes.push(byte);
        }
    }

    fn take_uart(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.uart_bytes)
    }

    fn reset(&mut self) -> Result<(), String> {
        self.cpu.reset().map_err(|error| error.to_string())?;
        self.board.set_pb5(false);
        self.led_on = false;
        self.pb5_high = false;
        self.pb5_millivolts = 0;
        self.led_microamps = 0;
        self.uart_bytes.clear();
        self.uart_rx.clear();
        Ok(())
    }

    fn cycles(&self) -> u64 {
        self.cpu.cycles()
    }
}

mod handle;
mod hex;
mod pacing;
#[cfg(test)]
mod tests;
mod worker;

pub use handle::SimHandle;
pub use hex::load_hex;
use worker::*;
