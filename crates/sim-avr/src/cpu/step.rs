use super::*;

impl Cpu {
    /// Builds a CPU with the provided program words.
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
        Ok(Self {
            circuit: builder.build(),
            datapath,
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
        })
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

    /// Executes one instruction and returns its trace.
    pub fn step(&mut self) -> Result<Option<AvrTrace>, SimError> {
        if self.halted {
            return Ok(None);
        }
        if self.sleeping && !self.wake_ready() {
            return Ok(None);
        }
        self.sleeping = false;
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
        Ok(Some(self.trace(fetched.instruction, cycles)))
    }

    pub(super) fn advance_serial(&mut self, cycles: u32) -> Result<(), SimError> {
        self.datapath.uart.tick(cycles);
        self.datapath.serial.spi.tick(cycles);
        self.datapath.serial.twi.tick(cycles);
        self.datapath.serial.adc.tick(cycles);
        let mask = self.uart_interrupt_mask();
        self.poll_uart(mask)
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

    /// Returns a register value.
    #[must_use]
    pub fn register(&self, index: u8) -> u8 {
        self.datapath
            .bank
            .read_byte(&self.circuit, usize::from(index))
            .unwrap_or(0)
    }

    /// Returns an I/O register value.
    #[must_use]
    pub fn io(&self, address: u8) -> u8 {
        self.io_read(address)
    }

    /// Returns a data space value.
    #[must_use]
    pub fn data(&self, address: u16) -> u8 {
        self.data[usize::from(address) & 0x0FFF]
    }

    /// Drives external pin inputs for port B, C, or D.
    pub fn set_pin_input(&mut self, port: u8, value: u8) -> Result<(), SimError> {
        self.drive_pin(port, value)?;
        let pin = self.gpio_pin(port)?;
        self.handle_pin_change(port, pin)
    }

    /// Returns the electrical state of one pin.
    pub fn pin_electrical(
        &self,
        port: u8,
        bit: u8,
        external: ExternalPin,
        config: &PinElectricalConfig,
    ) -> Result<PinElectrical, SimError> {
        let mode = self.gpio_ref(port)?.pin_mode(&self.circuit, bit)?;
        measure_pin(mode, external, config)
    }

    /// Applies an external voltage and updates the digital input when needed.
    pub fn set_pin_voltage(
        &mut self,
        port: u8,
        bit: u8,
        external: ExternalPin,
        config: &PinElectricalConfig,
    ) -> Result<PinElectrical, SimError> {
        let mode = self.gpio_ref(port)?.pin_mode(&self.circuit, bit)?;
        let state = measure_pin(mode, external, config)?;
        if mode.is_input() {
            self.set_pin_input_bit(port, bit, state.logic_level(config))?;
            let pin = self.gpio_pin(port)?;
            self.handle_pin_change(port, pin)?;
        }
        Ok(state)
    }

    /// Sets the pin-change mask for port B, C, or D.
    pub fn set_pin_change_mask(&mut self, port: u8, mask: u8) -> Result<(), SimError> {
        if port > 2 {
            return Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            ));
        }
        self.pin_change.pcmsk[usize::from(port)] = mask;
        Ok(())
    }

    /// Enables or disables pin-change interrupt for a port.
    pub fn enable_pin_change(&mut self, port: u8, enabled: bool) -> Result<(), SimError> {
        if port > 2 {
            return Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            ));
        }
        if enabled {
            self.pin_change.pcicr |= 1 << port;
        } else {
            self.pin_change.pcicr &= !(1 << port);
        }
        Ok(())
    }

    /// Returns the pin-change flag register.
    #[must_use]
    pub const fn pin_change_flags(&self) -> u8 {
        self.pin_change.pcifr
    }

    /// Clears one pin-change flag.
    pub fn clear_pin_change_flag(&mut self, port: u8) {
        if port <= 2 {
            self.pin_change.pcifr &= !(1 << port);
        }
    }

    pub(super) fn drive_pin(&mut self, port: u8, value: u8) -> Result<(), SimError> {
        match port {
            0 => self.datapath.gpio.b.set_pin_input(&mut self.circuit, value),
            1 => self.datapath.gpio.c.set_pin_input(&mut self.circuit, value),
            2 => self.datapath.gpio.d.set_pin_input(&mut self.circuit, value),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    pub(super) fn gpio_pin(&self, port: u8) -> Result<u8, SimError> {
        Ok(self.gpio_ref(port)?.read_pin(&self.circuit))
    }

    pub(super) fn gpio_ref(&self, port: u8) -> Result<&GpioPort, SimError> {
        match port {
            0 => Ok(&self.datapath.gpio.b),
            1 => Ok(&self.datapath.gpio.c),
            2 => Ok(&self.datapath.gpio.d),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    pub(super) fn set_pin_input_bit(
        &mut self,
        port: u8,
        bit: u8,
        value: bool,
    ) -> Result<(), SimError> {
        let Self {
            circuit, datapath, ..
        } = self;
        match port {
            0 => datapath.gpio.b.set_pin_input_bit(circuit, bit, value),
            1 => datapath.gpio.c.set_pin_input_bit(circuit, bit, value),
            2 => datapath.gpio.d.set_pin_input_bit(circuit, bit, value),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    pub(super) fn handle_pin_change(&mut self, port: u8, pin: u8) -> Result<(), SimError> {
        let index = usize::from(port);
        let changed = self.pin_change.last_pin[index] ^ pin;
        self.pin_change.last_pin[index] = pin;
        if changed & self.pin_change.pcmsk[index] == 0 || self.pin_change.pcicr & (1 << port) == 0 {
            return Ok(());
        }
        self.pin_change.pcifr |= 1 << port;
        self.request_interrupt(3 + port)
    }

    /// Returns active pull-up bits for port B, C, or D.
    #[must_use]
    pub fn pull_up(&self, port: u8) -> u8 {
        match port {
            0 => self.datapath.gpio.b.read_pull_up(&self.circuit),
            1 => self.datapath.gpio.c.read_pull_up(&self.circuit),
            2 => self.datapath.gpio.d.read_pull_up(&self.circuit),
            _ => 0,
        }
    }

    /// Returns the Timer0 PWM output compare A signal.
    #[must_use]
    pub const fn pwm_a(&self) -> bool {
        self.datapath.timers.timer0.pwm_output()
    }

    /// Returns the Timer1 core for configuration.
    pub fn timer1(&mut self) -> &mut Timer1 {
        &mut self.datapath.timers.timer1
    }

    /// Returns the Timer2 core for configuration.
    pub fn timer2(&mut self) -> &mut Timer2 {
        &mut self.datapath.timers.timer2
    }

    /// Returns the UART core for configuration.
    pub fn uart(&mut self) -> &mut Uart {
        &mut self.datapath.uart
    }

    /// Returns the SPI core for configuration.
    pub fn spi(&mut self) -> &mut Spi {
        &mut self.datapath.serial.spi
    }

    /// Returns the TWI core for configuration.
    pub fn twi(&mut self) -> &mut Twi {
        &mut self.datapath.serial.twi
    }

    /// Returns the ADC core for configuration.
    pub fn adc(&mut self) -> &mut Adc {
        &mut self.datapath.serial.adc
    }

    /// Requests SPI interrupt when SPIF is set.
    pub fn poll_spi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.spi.spif() {
            self.request_interrupt(17)?;
        }
        Ok(())
    }

    /// Requests TWI interrupt when TWINT is set.
    pub fn poll_twi(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.twi.twint() {
            self.request_interrupt(24)?;
        }
        Ok(())
    }

    /// Requests ADC interrupt when ADIF is set.
    pub fn poll_adc(&mut self) -> Result<(), SimError> {
        if self.datapath.serial.adc.flag() {
            self.request_interrupt(21)?;
        }
        Ok(())
    }

    /// Requests UART interrupts for the provided flag mask.
    pub fn poll_uart(&mut self, mask: u8) -> Result<(), SimError> {
        let uart = &self.datapath.uart;
        let rx = uart.rx_ready();
        let tx = uart.tx_complete();
        let udre = uart.tx_ready();
        self.update_uart_interrupt(18, rx, mask & 0x01 != 0)?;
        self.update_uart_interrupt(20, tx, mask & 0x02 != 0)?;
        self.update_uart_interrupt(19, udre, mask & 0x04 != 0)
    }

    pub(super) fn update_uart_interrupt(
        &mut self,
        index: u8,
        flag: bool,
        enabled: bool,
    ) -> Result<(), SimError> {
        if flag && enabled {
            return self.request_interrupt(index);
        }
        self.pending &= !(1u32 << index);
        Ok(())
    }

    /// Requests Timer1 interrupts for the provided flag mask.
    pub fn poll_timer1(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer1;
        let capture = timer.capture_flag();
        let compare_a = timer.compare_a_flag();
        let compare_b = timer.compare_b_flag();
        let overflow = timer.overflow_flag();
        if capture && mask & 0x01 != 0 {
            self.request_interrupt(10)?;
        }
        if compare_a && mask & 0x02 != 0 {
            self.request_interrupt(11)?;
        }
        if compare_b && mask & 0x04 != 0 {
            self.request_interrupt(12)?;
        }
        if overflow && mask & 0x08 != 0 {
            self.request_interrupt(13)?;
        }
        Ok(())
    }

    /// Requests Timer2 interrupts for the provided flag mask.
    pub fn poll_timer2(&mut self, mask: u8) -> Result<(), SimError> {
        let timer = &self.datapath.timers.timer2;
        let overflow = timer.overflow_flag();
        let compare = timer.compare_flag();
        if overflow && mask & 0x01 != 0 {
            self.request_interrupt(9)?;
        }
        if compare && mask & 0x02 != 0 {
            self.request_interrupt(7)?;
        }
        Ok(())
    }

    /// Returns the stack pointer.
    #[must_use]
    pub const fn sp(&self) -> u16 {
        self.sp
    }

    /// Requests an interrupt by vector index.
    pub fn request_interrupt(&mut self, index: u8) -> Result<(), SimError> {
        if usize::from(index) >= VECTORS.len() {
            return Err(SimError::InvalidNetlist(
                "interrupt vector out of range".to_string(),
            ));
        }
        self.pending |= 1 << index;
        Ok(())
    }

    /// Replaces the status register.
    pub fn set_sreg(&mut self, sreg: Sreg) {
        self.sreg = sreg;
    }

    pub(super) fn service_interrupt(&mut self) -> Result<bool, SimError> {
        if !self.sreg.interrupt || self.pending == 0 {
            return Ok(false);
        }
        let index = self.pending.trailing_zeros() as u8;
        self.pending &= !(1u32 << index);
        self.clear_interrupt_flag(index);
        if (3..=5).contains(&index) {
            self.pin_change.pcifr &= !(1 << (index - 3));
        }
        self.push_word(self.pc);
        self.sreg.interrupt = false;
        self.pc = vector_address(index);
        Ok(true)
    }

    pub(super) fn clear_interrupt_flag(&mut self, index: u8) {
        match index {
            14 => self.clear_timer0_compare_flag(),
            16 => self.clear_timer0_overflow_flag(),
            _ => {}
        }
    }

    pub(super) fn clear_timer0_overflow_flag(&mut self) {
        self.datapath.timers.timer0.take_overflow();
        self.io[0x35] &= !0x01;
    }

    pub(super) fn clear_timer0_compare_flag(&mut self) {
        self.datapath.timers.timer0.take_compare();
        self.io[0x35] &= !0x02;
    }

    pub(super) fn finish_sei(&mut self) {
        if self.sei_delay == 0 {
            return;
        }
        self.sei_delay -= 1;
        if self.sei_delay == 0 {
            self.sreg.interrupt = true;
        }
    }

    pub(super) fn instruction_cycles(&self, mnemonic: Mnemonic, next_pc: u16) -> u8 {
        if matches!(mnemonic, Mnemonic::Brbs | Mnemonic::Brbc) {
            return if self.pc != next_pc { 2 } else { 1 };
        }
        if is_skip(mnemonic) {
            if self.pc == next_pc {
                return 1;
            }
            let skipped = self.pc.wrapping_sub(next_pc).min(2);
            return 1 + skipped as u8;
        }
        base_cycles(mnemonic)
    }

    /// Returns the program counter.
    #[must_use]
    pub const fn pc(&self) -> u16 {
        self.pc
    }

    /// Returns the status register.
    #[must_use]
    pub const fn sreg(&self) -> Sreg {
        self.sreg
    }

    /// Returns true when halted.
    #[must_use]
    pub const fn is_halted(&self) -> bool {
        self.halted
    }

    /// Returns true when sleeping.
    #[must_use]
    pub const fn is_sleeping(&self) -> bool {
        self.sleeping
    }

    /// Returns total cycles since reset.
    #[must_use]
    pub const fn cycles(&self) -> u64 {
        self.cycles
    }

    pub(super) fn wake_ready(&self) -> bool {
        self.sreg.interrupt && self.pending != 0
    }

    pub(super) fn sleep(&mut self) -> Result<(), SimError> {
        self.sleeping = true;
        Ok(())
    }

    /// Resets the core to its initial state.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.circuit.reset()?;
        self.pc = 0;
        self.sreg = Sreg::default();
        self.halted = false;
        self.sp = 0x08FF;
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
