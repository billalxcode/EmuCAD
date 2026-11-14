use super::*;

impl Cpu {
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

    pub(crate) fn drive_pin(&mut self, port: u8, value: u8) -> Result<(), SimError> {
        match port {
            0 => self.datapath.gpio.b.set_pin_input(&mut self.circuit, value),
            1 => self.datapath.gpio.c.set_pin_input(&mut self.circuit, value),
            2 => self.datapath.gpio.d.set_pin_input(&mut self.circuit, value),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    pub(crate) fn gpio_pin(&self, port: u8) -> Result<u8, SimError> {
        Ok(self.gpio_ref(port)?.read_pin(&self.circuit))
    }

    pub(crate) fn gpio_ref(&self, port: u8) -> Result<&GpioPort, SimError> {
        match port {
            0 => Ok(&self.datapath.gpio.b),
            1 => Ok(&self.datapath.gpio.c),
            2 => Ok(&self.datapath.gpio.d),
            _ => Err(SimError::InvalidNetlist(
                "port index out of range".to_string(),
            )),
        }
    }

    pub(crate) fn set_pin_input_bit(
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

    pub(crate) fn handle_pin_change(&mut self, port: u8, pin: u8) -> Result<(), SimError> {
        let index = usize::from(port);
        let changed = self.pin_change.last_pin[index] ^ pin;
        self.pin_change.last_pin[index] = pin;
        if changed & self.pin_change.pcmsk[index] == 0 || self.pin_change.pcicr & (1 << port) == 0 {
            return Ok(());
        }
        self.pin_change.pcifr |= 1 << port;
        self.request_interrupt(3 + port)
    }
}
