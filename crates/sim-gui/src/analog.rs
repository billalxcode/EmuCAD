//! Background worker for analog DC and transient analysis.

use crate::model::Document;
use crate::netlist;
use crate::transient::{self, TransientSettings, TransientTrace};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Node voltages from one DC operating point.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DcResultView {
    pub voltages: Vec<(String, f64)>,
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
    Shutdown,
}

/// One completed analog analysis.
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

    /// Queues a transient request.
    pub fn request_transient(&mut self, document: Document, settings: TransientSettings) -> u64 {
        let request = self.allocate_request();
        let _ = self.commands.send(Command::Transient {
            request,
            document,
            settings,
        });
        request
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
            .recv_timeout(std::time::Duration::from_secs(2))
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

fn worker(commands: Receiver<Command>, events: Sender<Event>) {
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
            Command::Shutdown => break,
        }
    }
}

fn dc_result(document: &Document) -> Result<DcResultView, String> {
    let netlist = netlist::extract(document).map_err(|error| error.to_string())?;
    let result = netlist::solve_dc(&netlist)?;
    Ok(DcResultView {
        voltages: netlist::node_voltages(&netlist, &result),
        iterations: result.iterations,
        residual: result.residual,
    })
}

#[cfg(test)]
pub(crate) mod tests;
