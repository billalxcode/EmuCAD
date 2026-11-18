use super::*;

pub(super) fn worker(commands: Receiver<Command>, events: Sender<Event>) {
    let mut state = WorkerState::new();
    loop {
        if state.running {
            step_chunk(&mut state, &events);
            state.pace();
            if !drain_commands(&commands, &mut state, &events) {
                break;
            }
            continue;
        }
        if !wait_command(&commands, &mut state, &events) {
            break;
        }
    }
}

pub(super) fn wait_command(
    commands: &Receiver<Command>,
    state: &mut WorkerState,
    events: &Sender<Event>,
) -> bool {
    match commands.recv_timeout(Duration::from_millis(POLL_MS)) {
        Ok(Command::Shutdown) => false,
        Ok(command) => {
            handle_command(command, state, events);
            true
        }
        Err(RecvTimeoutError::Timeout) => true,
        Err(RecvTimeoutError::Disconnected) => false,
    }
}

pub(super) fn drain_commands(
    commands: &Receiver<Command>,
    state: &mut WorkerState,
    events: &Sender<Event>,
) -> bool {
    loop {
        match commands.try_recv() {
            Ok(Command::Shutdown) => return false,
            Ok(command) => handle_command(command, state, events),
            Err(TryRecvError::Empty) => return true,
            Err(TryRecvError::Disconnected) => return false,
        }
    }
}

pub(super) fn handle_command(command: Command, state: &mut WorkerState, events: &Sender<Event>) {
    match command {
        Command::Load(words) => state.load(words, events),
        Command::Run => state.run(true, events),
        Command::Pause => state.run(false, events),
        Command::Step => state.step(events),
        Command::Reset => state.reset(events),
        Command::Uart(data) => state.queue_uart(data),
        Command::SampleEvery(every) => state.sample_every = every.max(1),
        Command::Shutdown => {}
    }
}

pub(super) fn update_running(
    machine: &Option<SimMachine>,
    running: &mut bool,
    value: bool,
    events: &Sender<Event>,
) {
    *running = value && machine.is_some();
    send_status(machine, *running, events);
}

pub(super) fn queue_uart(machine: &mut Option<SimMachine>, data: Vec<u8>) {
    if let Some(machine) = machine {
        machine.queue_uart(&data);
    }
}

pub(super) fn load_machine(
    words: Vec<u16>,
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    match SimMachine::new(words) {
        Ok(next) => install_machine(next, machine, last, events),
        Err(error) => {
            *machine = None;
            let _ = events.send(Event::Error(error));
        }
    }
    send_status(machine, false, events);
}

pub(super) fn install_machine(
    next: SimMachine,
    machine: &mut Option<SimMachine>,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *last = SampleState::of(&next);
    *machine = Some(next);
    let _ = events.send(Event::UartCleared);
    send_sample(0, *last, events);
}

pub(super) fn step_once(
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    if let Some(machine) = machine {
        handle_step(machine, last, events);
        send_uart(machine, events);
    }
    send_status(machine, false, events);
}

pub(super) fn handle_step(
    machine: &mut SimMachine,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    match machine.step() {
        Ok(true) => record_step(machine, last, events),
        Ok(false) => {}
        Err(error) => {
            let _ = events.send(Event::Error(error));
        }
    }
}

pub(super) fn record_step(machine: &SimMachine, last: &mut SampleState, events: &Sender<Event>) {
    let state = SampleState::of(machine);
    if state == *last {
        return;
    }
    *last = state;
    send_sample(machine.cycles(), state, events);
}

pub(super) fn reset_machine(
    machine: &mut Option<SimMachine>,
    running: &mut bool,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    *running = false;
    if let Some(machine) = machine {
        reset_current(machine, last, events);
    }
    send_status(machine, false, events);
}

pub(super) fn reset_current(
    machine: &mut SimMachine,
    last: &mut SampleState,
    events: &Sender<Event>,
) {
    match machine.reset() {
        Ok(()) => {
            *last = SampleState::default();
            let _ = events.send(Event::UartCleared);
            send_sample(0, *last, events);
        }
        Err(error) => {
            let _ = events.send(Event::Error(error));
        }
    }
}

pub(super) fn step_chunk(state: &mut WorkerState, events: &Sender<Event>) {
    let Some(current) = state.machine.as_mut() else {
        state.running = false;
        return;
    };
    advance(
        current,
        &mut state.running,
        &mut state.last,
        state.sample_every,
        events,
    );
    send_uart(current, events);
    send_status(&state.machine, state.running, events);
}

pub(super) fn advance(
    machine: &mut SimMachine,
    running: &mut bool,
    last: &mut SampleState,
    sample_every: u64,
    events: &Sender<Event>,
) {
    for _ in 0..CHUNK {
        if !step_one(machine, running, events) {
            break;
        }
        sample_step(machine, last, sample_every, events);
    }
}

pub(super) fn step_one(
    machine: &mut SimMachine,
    running: &mut bool,
    events: &Sender<Event>,
) -> bool {
    match machine.step() {
        Ok(true) => true,
        Ok(false) => {
            *running = false;
            false
        }
        Err(error) => {
            let _ = events.send(Event::Error(error));
            *running = false;
            false
        }
    }
}

pub(super) fn sample_step(
    machine: &SimMachine,
    last: &mut SampleState,
    sample_every: u64,
    events: &Sender<Event>,
) {
    let state = SampleState::of(machine);
    let cycle = machine.cycles();
    let every = sample_every.max(1);
    if state == *last && cycle % every != 0 {
        return;
    }
    *last = state;
    send_sample(cycle, state, events);
}

pub(super) fn send_uart(machine: &mut SimMachine, events: &Sender<Event>) {
    let bytes = machine.take_uart();
    if !bytes.is_empty() {
        let _ = events.send(Event::Uart(bytes));
    }
}

pub(super) fn send_status(machine: &Option<SimMachine>, running: bool, events: &Sender<Event>) {
    let status = match machine {
        Some(machine) => Status {
            cycles: machine.cycles(),
            pb5_high: machine.pb5_high,
            pb5_millivolts: machine.pb5_millivolts,
            led_on: machine.led_on,
            led_microamps: machine.led_microamps,
            running,
            loaded: true,
        },
        None => Status::default(),
    };
    let _ = events.send(Event::Status(status));
}

pub(super) fn send_sample(cycle: u64, state: SampleState, events: &Sender<Event>) {
    let _ = events.send(Event::Sample(state.sample(cycle)));
}
