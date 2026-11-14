use super::*;

impl Cpu {
    pub fn register(&self, index: u8) -> u8 {
        self.registers.get(usize::from(index)).copied().unwrap_or(0)
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
}

impl Cpu {
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
}

impl Cpu {
    /// Replaces the status register.
    pub fn set_sreg(&mut self, sreg: Sreg) {
        self.sreg = sreg;
    }
}
