//! Background worker for mixed AVR and analog simulation.

use crate::model::Document;
use crate::netlist::{self, MixedPinPlan};
use crate::transient::TransientSettings;
use ::mixed::AvrSession;
use analysis::TransientConfig;
use kernel::{NodeId, Time};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

const CHUNK: usize = 16;

/// One mixed simulation sample.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MixedSample {
    pub time: Time,
    pub cycles: u64,
    pub outputs: Vec<(u8, f64)>,
    pub adcs: Vec<f64>,
    pub pwms: Vec<f64>,
    pub probes: Vec<f64>,
}

/// Trace collected by the mixed worker.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct MixedTrace {
    pub outputs: Vec<u8>,
    pub adc_channels: Vec<u8>,
    pub pwm_pins: Vec<u8>,
    pub probes: Vec<String>,
    pub samples: Vec<MixedSample>,
}

struct StartPayload {
    document: Document,
    settings: TransientSettings,
    firmware: Vec<u16>,
    plan: MixedPinPlan,
}

enum Command {
    Start {
        request: u64,
        payload: Box<StartPayload>,
    },
    Pause,
    Step,
    Reset,
    Shutdown,
}

/// One mixed worker event.
#[derive(Clone, Debug)]
pub(crate) enum Event {
    Started {
        request: u64,
        outputs: Vec<u8>,
        adc_channels: Vec<u8>,
        pwm_pins: Vec<u8>,
        probes: Vec<String>,
        steps: usize,
    },
    Samples {
        request: u64,
        samples: Vec<MixedSample>,
    },
    Paused {
        request: u64,
    },
    Finished {
        request: u64,
    },
    Reset {
        request: u64,
        outputs: Vec<u8>,
        adc_channels: Vec<u8>,
        pwm_pins: Vec<u8>,
        probes: Vec<String>,
        steps: usize,
    },
    Error {
        request: u64,
        error: String,
    },
}

/// Handle to the mixed worker thread.
pub(crate) struct MixedHandle {
    commands: Sender<Command>,
    events: Receiver<Event>,
    next_request: u64,
    join: Option<JoinHandle<()>>,
}

impl MixedHandle {
    /// Starts the mixed worker.
    #[must_use]
    pub fn spawn() -> Self {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let join = thread::spawn(move || worker(command_rx, event_tx));
        Self {
            commands,
            events,
            next_request: 0,
            join: Some(join),
        }
    }

    /// Starts a mixed session.
    pub fn start(
        &mut self,
        document: Document,
        settings: TransientSettings,
        firmware: Vec<u16>,
        plan: MixedPinPlan,
    ) -> u64 {
        let request = self.allocate_request();
        let _ = self.commands.send(Command::Start {
            request,
            payload: Box::new(StartPayload {
                document,
                settings,
                firmware,
                plan,
            }),
        });
        request
    }

    /// Pauses the active session.
    pub fn pause(&self) {
        let _ = self.commands.send(Command::Pause);
    }

    /// Advances a paused session by one step.
    pub fn step(&self) {
        let _ = self.commands.send(Command::Step);
    }

    /// Resets the session to time zero.
    pub fn reset(&self) {
        let _ = self.commands.send(Command::Reset);
    }

    /// Drains completed events without blocking.
    #[must_use]
    pub fn poll(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
            if events.len() >= 64 {
                break;
            }
        }
        events
    }

    fn allocate_request(&mut self) -> u64 {
        let request = self.next_request;
        self.next_request = self.next_request.wrapping_add(1);
        request
    }

    #[cfg(test)]
    pub(crate) fn wait(&self) -> Event {
        self.events
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("mixed event")
    }
}

impl Drop for MixedHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct SessionState {
    request: u64,
    document: Document,
    settings: TransientSettings,
    firmware: Vec<u16>,
    plan: MixedPinPlan,
    probes: Vec<(String, NodeId)>,
    adc_channels: Vec<u8>,
    pwm_pins: Vec<u8>,
    session: AvrSession,
    paused: bool,
    finished: bool,
    steps: usize,
}

fn worker(commands: Receiver<Command>, events: Sender<Event>) {
    let mut state = None;
    while let Ok(command) = commands.recv() {
        match command {
            Command::Start { request, payload } => {
                state = start_session(request, *payload, &events);
                if let Some(active) = state.as_mut() {
                    drive_session(active, &commands, &events);
                }
            }
            Command::Pause => pause_session(state.as_mut(), &events),
            Command::Step => step_session(state.as_mut(), &events),
            Command::Reset => reset_session(state.as_mut(), &events),
            Command::Shutdown => break,
        }
    }
}

fn start_session(
    request: u64,
    payload: StartPayload,
    events: &Sender<Event>,
) -> Option<SessionState> {
    match build_state(request, payload) {
        Ok(state) => {
            let _ = events.send(started_event(&state));
            Some(state)
        }
        Err(error) => {
            let _ = events.send(Event::Error { request, error });
            None
        }
    }
}

fn started_event(state: &SessionState) -> Event {
    Event::Started {
        request: state.request,
        outputs: state.plan.outputs.clone(),
        adc_channels: state.adc_channels.clone(),
        pwm_pins: state.pwm_pins.clone(),
        probes: state.plan.probes.clone(),
        steps: state.settings.steps,
    }
}

fn reset_event(state: &SessionState) -> Event {
    Event::Reset {
        request: state.request,
        outputs: state.plan.outputs.clone(),
        adc_channels: state.adc_channels.clone(),
        pwm_pins: state.pwm_pins.clone(),
        probes: state.plan.probes.clone(),
        steps: state.settings.steps,
    }
}

struct ResolvedSession {
    probes: Vec<(String, NodeId)>,
    adc_channels: Vec<u8>,
    pwm_pins: Vec<u8>,
    session: AvrSession,
}

impl SessionState {
    fn new(request: u64, payload: StartPayload, resolved: ResolvedSession) -> Self {
        Self {
            request,
            document: payload.document,
            settings: payload.settings,
            firmware: payload.firmware,
            plan: payload.plan,
            probes: resolved.probes,
            adc_channels: resolved.adc_channels,
            pwm_pins: resolved.pwm_pins,
            session: resolved.session,
            paused: false,
            finished: false,
            steps: 0,
        }
    }
}

fn build_state(request: u64, payload: StartPayload) -> Result<SessionState, String> {
    let netlist = netlist::extract(&payload.document).map_err(|error| error.to_string())?;
    let config = mixed_config(&payload.settings);
    let resolved = resolve_session(&netlist, &payload.plan, payload.firmware.clone(), &config)?;
    Ok(SessionState::new(request, payload, resolved))
}

fn resolve_session(
    netlist: &netlist::Netlist,
    plan: &MixedPinPlan,
    firmware: Vec<u16>,
    config: &TransientConfig,
) -> Result<ResolvedSession, String> {
    let probes = netlist::probe_nodes(netlist, &plan.probes)?;
    let adc_channels = plan.adcs.iter().map(|(channel, _)| *channel).collect();
    let pwm_pins = plan.pwms.clone();
    let session = netlist::build_mixed_session(netlist, firmware, plan, config)?;
    Ok(ResolvedSession {
        probes,
        adc_channels,
        pwm_pins,
        session,
    })
}

fn mixed_config(settings: &TransientSettings) -> TransientConfig {
    TransientConfig {
        dt: settings.dt,
        steps: settings.steps,
        gmin: settings.gmin,
        use_dc_op: settings.use_dc_op,
        ..Default::default()
    }
}

fn drive_session(state: &mut SessionState, commands: &Receiver<Command>, events: &Sender<Event>) {
    while !state.paused && !state.finished {
        run_chunk(state, events);
        if state.finished {
            let _ = events.send(Event::Finished {
                request: state.request,
            });
            break;
        }
        match commands.try_recv() {
            Ok(Command::Pause) => state.paused = true,
            Ok(Command::Step) => {
                state.paused = true;
                step_session(Some(state), events);
            }
            Ok(Command::Reset) => {
                reset_session(Some(state), events);
                state.paused = true;
            }
            Ok(Command::Shutdown) => state.finished = true,
            Ok(_) => {}
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => state.finished = true,
        }
        if state.paused {
            let _ = events.send(Event::Paused {
                request: state.request,
            });
        }
    }
}

fn run_chunk(state: &mut SessionState, events: &Sender<Event>) {
    let mut samples = Vec::new();
    for _ in 0..CHUNK {
        if state.steps >= state.settings.steps {
            state.finished = true;
            break;
        }
        match state.session.step() {
            Ok(_) => samples.push(sample_state(state)),
            Err(error) => {
                let _ = events.send(Event::Error {
                    request: state.request,
                    error: error.to_string(),
                });
                state.finished = true;
                break;
            }
        }
        state.steps += 1;
    }
    if !samples.is_empty() {
        let _ = events.send(Event::Samples {
            request: state.request,
            samples,
        });
    }
}

fn sample_state(state: &SessionState) -> MixedSample {
    let outputs = state
        .plan
        .outputs
        .iter()
        .enumerate()
        .map(|(index, &pin)| (pin, state.session.node_voltage(index).unwrap_or(f64::NAN)))
        .collect();
    let adcs = sample_adcs(state);
    let pwms = sample_pwms(state);
    let probes = sample_probes(state);
    MixedSample {
        time: state.session.time(),
        cycles: state.session.cpu().cycles(),
        outputs,
        adcs,
        pwms,
        probes,
    }
}

fn sample_adcs(state: &SessionState) -> Vec<f64> {
    (0..state.adc_channels.len())
        .map(|index| state.session.adc_voltage_at(index).unwrap_or(f64::NAN))
        .collect()
}

fn sample_pwms(state: &SessionState) -> Vec<f64> {
    (0..state.pwm_pins.len())
        .map(|index| state.session.pwm_voltage_at(index).unwrap_or(f64::NAN))
        .collect()
}

fn sample_probes(state: &SessionState) -> Vec<f64> {
    state
        .probes
        .iter()
        .map(|(_, node)| state.session.analog_node_voltage(*node).unwrap_or(f64::NAN))
        .collect()
}

fn pause_session(state: Option<&mut SessionState>, events: &Sender<Event>) {
    let Some(state) = state else {
        return;
    };
    state.paused = true;
    let _ = events.send(Event::Paused {
        request: state.request,
    });
}

fn step_session(state: Option<&mut SessionState>, events: &Sender<Event>) {
    let Some(state) = state else {
        return;
    };
    if state.finished {
        return;
    }
    match state.session.step() {
        Ok(_) => {
            let sample = sample_state(state);
            let _ = events.send(Event::Samples {
                request: state.request,
                samples: vec![sample],
            });
        }
        Err(error) => {
            let _ = events.send(Event::Error {
                request: state.request,
                error: error.to_string(),
            });
        }
    }
    state.steps += 1;
    if state.steps >= state.settings.steps {
        state.finished = true;
        let _ = events.send(Event::Finished {
            request: state.request,
        });
    }
}

fn reset_session(state: Option<&mut SessionState>, events: &Sender<Event>) {
    let Some(state) = state else {
        return;
    };
    let payload = StartPayload {
        document: state.document.clone(),
        settings: state.settings,
        firmware: state.firmware.clone(),
        plan: state.plan.clone(),
    };
    match build_state(state.request, payload) {
        Ok(mut fresh) => {
            fresh.paused = true;
            let event = reset_event(&fresh);
            *state = fresh;
            let _ = events.send(event);
            let _ = events.send(Event::Paused {
                request: state.request,
            });
        }
        Err(error) => {
            let _ = events.send(Event::Error {
                request: state.request,
                error,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, MixedHandle};
    use crate::model::Document;
    use crate::netlist::MixedPinPlan;
    use crate::transient::TransientSettings;

    #[test]
    fn missing_schematic_reports_error() {
        let mut worker = MixedHandle::spawn();
        let request = worker.start(
            Document::default(),
            TransientSettings::default(),
            vec![0x0000],
            empty_plan(),
        );
        match worker.wait() {
            Event::Error {
                request: event_request,
                error,
            } => {
                assert_eq!(event_request, request);
                assert!(!error.is_empty());
            }
            _ => panic!("expected mixed error"),
        }
    }

    fn empty_plan() -> MixedPinPlan {
        MixedPinPlan {
            outputs: Vec::new(),
            adcs: Vec::new(),
            pwms: Vec::new(),
            uart_tx: None,
            uart_rx: None,
            high: 5.0,
            low: 0.0,
            external_resistance: 1000.0,
            adc_reference: 5.0,
            uart_threshold: 2.5,
            probes: Vec::new(),
        }
    }
}
