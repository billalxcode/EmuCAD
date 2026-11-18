use super::worker::worker;
use super::*;

pub struct SimHandle {
    commands: Sender<Command>,
    events: Receiver<Event>,
    join: Option<JoinHandle<()>>,
}

impl SimHandle {
    /// Starts the simulation worker.
    #[must_use]
    pub fn spawn() -> Self {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let join = thread::spawn(move || worker(command_rx, event_tx));
        Self {
            commands,
            events,
            join: Some(join),
        }
    }

    /// Loads a program and pauses the worker.
    pub fn load(&self, words: Vec<u16>) -> Result<(), String> {
        self.send(Command::Load(words))
    }

    /// Starts continuous execution.
    pub fn run(&self) -> Result<(), String> {
        self.send(Command::Run)
    }

    /// Pauses continuous execution.
    pub fn pause(&self) -> Result<(), String> {
        self.send(Command::Pause)
    }

    /// Executes one instruction.
    pub fn step(&self) -> Result<(), String> {
        self.send(Command::Step)
    }

    /// Resets the current program.
    pub fn reset(&self) -> Result<(), String> {
        self.send(Command::Reset)
    }

    /// Queues bytes for the simulated UART receiver.
    pub fn send_uart(&self, data: Vec<u8>) -> Result<(), String> {
        self.send(Command::Uart(data))
    }

    /// Sets the probe sampling interval in CPU cycles.
    pub fn set_sample_every(&self, every: u64) -> Result<(), String> {
        self.send(Command::SampleEvery(every))
    }

    /// Drains pending events.
    #[must_use]
    pub fn poll(&self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            events.push(event);
            if events.len() >= 4096 {
                break;
            }
        }
        events
    }

    #[cfg(test)]
    pub(super) fn wait_status(&self) -> Option<Status> {
        loop {
            match self.events.recv_timeout(Duration::from_secs(1)) {
                Ok(Event::Status(status)) => return Some(status),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    }

    #[cfg(test)]
    pub(super) fn wait_until(&self, predicate: impl Fn(&Status) -> bool) -> Option<Status> {
        loop {
            let status = self.wait_status()?;
            if predicate(&status) {
                return Some(status);
            }
        }
    }

    fn send(&self, command: Command) -> Result<(), String> {
        self.commands
            .send(command)
            .map_err(|_| "simulation worker stopped".to_string())
    }
}

impl Drop for SimHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
