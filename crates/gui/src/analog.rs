//! Background worker for analog DC and transient analysis.

use crate::model::Document;
use crate::netlist;
use crate::transient::{self, TraceMetadata, TransientSettings, TransientTrace};
use analysis::{TransientSample, TransientSession};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};

const CHUNK: usize = 256;

/// Node voltages from one DC operating point.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DcResultView {
    pub voltages: Vec<(String, f64)>,
    pub currents: Vec<(String, Option<f64>)>,
    pub iterations: usize,
    pub residual: f64,
}

enum Command {
    Dc {
        request: u64,
        document: Document,
    },
    Transient {
        request: u64,
        document: Document,
        settings: TransientSettings,
    },
    StartTransient {
        request: u64,
        document: Document,
        settings: TransientSettings,
    },
    Pause,
    Step,
    Reset,
    Shutdown,
}

/// One analog worker event.
#[derive(Clone, Debug)]
pub(crate) enum Event {
    Dc {
        request: u64,
        result: Result<DcResultView, String>,
    },
    Transient {
        request: u64,
        result: Result<TransientTrace, String>,
    },
    TransientStarted {
        request: u64,
        metadata: TraceMetadata,
    },
    TransientSamples {
        request: u64,
        samples: Vec<TransientSample>,
    },
    TransientPaused {
        request: u64,
    },
    TransientFinished {
        request: u64,
    },
    TransientReset {
        request: u64,
    },
    TransientError {
        request: u64,
        error: String,
    },
}

/// Handle to the analog worker thread.
pub(crate) struct AnalogHandle {
    commands: Sender<Command>,
    events: Receiver<Event>,
    next_request: u64,
    join: Option<JoinHandle<()>>,
}

impl AnalogHandle {
    /// Starts the analog worker.
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

    /// Queues a DC operating point request.
    pub fn request_dc(&mut self, document: Document) -> u64 {
        let request = self.allocate_request();
        let _ = self.commands.send(Command::Dc { request, document });
        request
    }

    /// Queues a complete transient request.
    pub fn request_transient(&mut self, document: Document, settings: TransientSettings) -> u64 {
        let request = self.allocate_request();
        let _ = self.commands.send(Command::Transient {
            request,
            document,
            settings,
        });
        request
    }

    /// Starts an incremental transient session.
    pub fn start_transient(&mut self, document: Document, settings: TransientSettings) -> u64 {
        let request = self.allocate_request();
        let _ = self.commands.send(Command::StartTransient {
            request,
            document,
            settings,
        });
        request
    }

    /// Pauses the active transient session.
    pub fn pause(&self) {
        let _ = self.commands.send(Command::Pause);
    }

    /// Advances the paused transient session by one step.
    pub fn step(&self) {
        let _ = self.commands.send(Command::Step);
    }

    /// Resets the transient session to time zero.
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
            .expect("analog event")
    }
}

impl Drop for AnalogHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct SessionState {
    request: u64,
    session: TransientSession,
    paused: bool,
    finished: bool,
}

fn worker(commands: Receiver<Command>, events: Sender<Event>) {
    let mut session = None;
    while let Ok(command) = commands.recv() {
        match command {
            Command::Dc { request, document } => {
                let result = dc_result(&document);
                let _ = events.send(Event::Dc { request, result });
            }
            Command::Transient {
                request,
                document,
                settings,
            } => {
                let result = transient::solve(&document, &settings);
                let _ = events.send(Event::Transient { request, result });
            }
            Command::StartTransient {
                request,
                document,
                settings,
            } => {
                session = start_session(request, &document, settings, &events);
                if let Some(state) = session.as_mut() {
                    drive_session(state, &commands, &events);
                }
            }
            Command::Pause => pause_session(session.as_mut(), &events),
            Command::Step => step_session(session.as_mut(), &events),
            Command::Reset => reset_session(session.as_mut(), &events),
            Command::Shutdown => break,
        }
    }
}

fn start_session(
    request: u64,
    document: &Document,
    settings: TransientSettings,
    events: &Sender<Event>,
) -> Option<SessionState> {
    match build_session(document, &settings) {
        Ok((session, metadata)) => {
            let _ = events.send(Event::TransientStarted { request, metadata });
            Some(SessionState {
                request,
                session,
                paused: false,
                finished: false,
            })
        }
        Err(error) => {
            let _ = events.send(Event::TransientError { request, error });
            None
        }
    }
}

fn build_session(
    document: &Document,
    settings: &TransientSettings,
) -> Result<(TransientSession, TraceMetadata), String> {
    let netlist = netlist::extract(document).map_err(|error| error.to_string())?;
    let metadata = transient::metadata(&netlist, settings);
    let circuit = netlist::build_circuit(&netlist)?;
    let config = transient::fixed_config(settings)?;
    let session = TransientSession::new(&circuit, &config).map_err(|error| error.to_string())?;
    Ok((session, metadata))
}

fn drive_session(state: &mut SessionState, commands: &Receiver<Command>, events: &Sender<Event>) {
    while !state.paused && !state.finished {
        run_chunk(state, events);
        if state.finished {
            let _ = events.send(Event::TransientFinished {
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
            let _ = events.send(Event::TransientPaused {
                request: state.request,
            });
        }
    }
}

fn run_chunk(state: &mut SessionState, events: &Sender<Event>) {
    let mut samples = Vec::new();
    for _ in 0..CHUNK {
        if state.session.is_finished() {
            state.finished = true;
            break;
        }
        match state.session.step() {
            Ok(sample) => samples.push(sample),
            Err(error) => {
                let _ = events.send(Event::TransientError {
                    request: state.request,
                    error: error.to_string(),
                });
                state.finished = true;
                break;
            }
        }
    }
    if !samples.is_empty() {
        let _ = events.send(Event::TransientSamples {
            request: state.request,
            samples,
        });
    }
}

fn pause_session(state: Option<&mut SessionState>, events: &Sender<Event>) {
    let Some(state) = state else {
        return;
    };
    state.paused = true;
    let _ = events.send(Event::TransientPaused {
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
        Ok(sample) => {
            let _ = events.send(Event::TransientSamples {
                request: state.request,
                samples: vec![sample],
            });
        }
        Err(error) => {
            let _ = events.send(Event::TransientError {
                request: state.request,
                error: error.to_string(),
            });
        }
    }
    if state.session.is_finished() {
        state.finished = true;
        let _ = events.send(Event::TransientFinished {
            request: state.request,
        });
    }
}

fn reset_session(state: Option<&mut SessionState>, events: &Sender<Event>) {
    let Some(state) = state else {
        return;
    };
    match state.session.reset() {
        Ok(()) => {
            state.paused = true;
            state.finished = false;
            let _ = events.send(Event::TransientReset {
                request: state.request,
            });
        }
        Err(error) => {
            let _ = events.send(Event::TransientError {
                request: state.request,
                error: error.to_string(),
            });
        }
    }
}

fn dc_result(document: &Document) -> Result<DcResultView, String> {
    let netlist = netlist::extract(document).map_err(|error| error.to_string())?;
    let result = netlist::solve_dc(&netlist)?;
    let currents = netlist::device_currents(&netlist, &result)?;
    Ok(DcResultView {
        voltages: netlist::node_voltages(&netlist, &result),
        currents,
        iterations: result.iterations,
        residual: result.residual,
    })
}

#[cfg(test)]
pub(crate) mod tests;
