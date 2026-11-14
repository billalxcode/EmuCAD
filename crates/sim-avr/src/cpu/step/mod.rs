use super::*;

impl Cpu {
    pub fn new(program: Vec<u16>) -> Result<Self, SimError> {
        if program.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR program must not be empty".to_string(),
            ));
        }
        Self::build(program)
    }

    pub(super) fn build(program: Vec<u16>) -> Result<Self, SimError> {
        let mut builder = DigitalBuilder::new();
        let datapath = build_datapath(&mut builder)?;
        Ok(Self::assemble(builder.build(), datapath, program))
    }

    fn assemble(circuit: DigitalCircuit, datapath: Datapath, program: Vec<u16>) -> Self {
        Self {
            circuit,
            datapath,
            registers: [0; 32],
            program: ProgramMemory::new(program),
            pc: 0,
            sreg: Sreg::default(),
            halted: false,
            io: [0; 256],
            data: vec![0; 0x1000],
            sp: 0x08FF,
            pending: 0,
            sei_delay: 0,
            sleeping: false,
            cycles: 0,
            pin_change: PinChange::default(),
        }
    }

    /// Replaces program memory and resets the core.
    pub fn load_program(&mut self, program: Vec<u16>) -> Result<(), SimError> {
        if program.is_empty() {
            return Err(SimError::InvalidNetlist(
                "AVR program must not be empty".to_string(),
            ));
        }
        self.program = ProgramMemory::new(program);
        self.reset()
    }

    /// Returns the current program words.
    #[must_use]
    pub fn program(&self) -> &[u16] {
        self.program.words()
    }

    /// Returns static digital circuit size information.
    #[must_use]
    pub fn circuit_stats(&self) -> sim_cells::CircuitStats {
        self.circuit.stats()
    }

    /// Executes one instruction and returns its trace.
    #[inline]
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        Ok(self.step_inner(true)?.and_then(|outcome| outcome.trace))
    }

    /// Executes one instruction without building a trace.
    #[inline]
    pub fn step_untraced(&mut self) -> Result<Option<u8>, SimError> {
        Ok(self.step_inner(false)?.map(|outcome| outcome.cycles))
    }

    #[inline]
    fn step_inner(&mut self, record: bool) -> Result<Option<StepOutcome>, SimError> {
        if !self.ready()? {
            return Ok(None);
        }
        let interrupt = self.service_interrupt()?;
        let fetched = self.program.fetch(self.pc)?;
        let next_pc = self.pc.wrapping_add(u16::from(fetched.words));
        self.execute(&fetched.instruction, next_pc)?;
        self.finish_sei();
        let cycles = self.instruction_cycles(fetched.instruction.mnemonic, next_pc);
        let total = u64::from(cycles) + if interrupt { 4 } else { 0 };
        self.cycles += total;
        self.advance_timer0(total as u32)?;
        self.advance_serial(total as u32)?;
        let trace = record.then(|| self.trace(fetched.instruction, cycles));
        Ok(Some(StepOutcome { trace, cycles }))
    }

    fn ready(&mut self) -> Result<bool, SimError> {
        if self.halted {
            return Ok(false);
        }
        if self.sleeping && !self.wake_ready() {
            return Ok(false);
        }
        self.sleeping = false;
        Ok(true)
    }

    #[inline]
    pub(super) fn advance_serial(&mut self, cycles: u32) -> Result<(), SimError> {
        let mask = self.uart_interrupt_mask();
        if mask == 0 && !self.serial_busy() {
            return Ok(());
        }
        self.datapath.uart.tick(cycles);
        self.datapath.serial.spi.tick(cycles);
        self.datapath.serial.twi.tick(cycles);
        self.datapath.serial.adc.tick(cycles);
        self.poll_uart(mask)
    }

    fn serial_busy(&self) -> bool {
        self.datapath.uart.tx_active()
            || self.datapath.serial.spi.busy()
            || self.datapath.serial.twi.busy()
            || self.datapath.serial.adc.busy()
    }

    pub(super) fn uart_interrupt_mask(&self) -> u8 {
        let enable = self.io[0xC1];
        let mut mask = 0;
        if enable & 0x80 != 0 {
            mask |= 0x01;
        }
        if enable & 0x40 != 0 {
            mask |= 0x02;
        }
        if enable & 0x20 != 0 {
            mask |= 0x04;
        }
        mask
    }

    /// Executes until halt or the step limit.
    pub fn run(&mut self, max_steps: usize) -> Result<Vec<AvrTrace>, SimError> {
        let mut traces = Vec::new();
        for _ in 0..max_steps {
            match self.step()? {
                Some(trace) => traces.push(trace),
                None => return Ok(traces),
            }
        }
        Err(SimError::ConvergenceFailure {
            iteration: max_steps,
        })
    }

    /// Executes until halt or the step limit without recording traces.
    pub fn run_untraced(&mut self, max_steps: usize) -> Result<u64, SimError> {
        for step in 0..max_steps {
            if self.step_untraced()?.is_none() {
                return Ok(step as u64);
            }
        }
        Err(SimError::ConvergenceFailure {
            iteration: max_steps,
        })
    }

    /// Resets the core to its initial state.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.circuit.reset()?;
        self.pc = 0;
        self.sreg = Sreg::default();
        self.halted = false;
        self.sp = 0x08FF;
        self.registers.fill(0);
        self.io.fill(0);
        self.data.fill(0);
        self.pending = 0;
        self.sei_delay = 0;
        self.sleeping = false;
        self.cycles = 0;
        self.pin_change = PinChange::default();
        self.datapath.timers.timer0.reset()?;
        Ok(())
    }
}

mod interrupts;
mod peripherals;
mod pins;
mod registers;
