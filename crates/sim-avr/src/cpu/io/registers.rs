use super::*;

impl Cpu {
    pub(crate) fn read_pointer(&self, low: u8) -> u16 {
        u16::from(self.register(low)) | (u16::from(self.register(low + 1)) << 8)
    }

    pub(crate) fn write_pointer(&mut self, low: u8, value: u16) -> Result<(), SimError> {
        self.write_register(low, value as u8)?;
        self.write_register(low + 1, (value >> 8) as u8)
    }

    pub(crate) fn read_data(&mut self, address: u16) -> u8 {
        if (0x20..0x100).contains(&address) {
            return self.io_read_mut(address as u8);
        }
        self.data[usize::from(address) & 0x0FFF]
    }

    pub(crate) fn io_read(&self, address: u8) -> u8 {
        if let Some(value) = self.uart_read_register(address) {
            return value;
        }
        if let Some(value) = self.timer0_read(address) {
            return value;
        }
        if let Some(value) = self.gpio_read(address) {
            return value;
        }
        if address == 0x3F {
            return self.sreg.to_byte();
        }
        self.io.get(usize::from(address)).copied().unwrap_or(0)
    }

    pub(crate) fn io_read_mut(&mut self, address: u8) -> u8 {
        if let Some(value) = self.uart_read_register_mut(address) {
            return value;
        }
        self.io_read(address)
    }

    pub(crate) fn uart_read_register(&self, address: u8) -> Option<u8> {
        match address {
            0xC0 => Some(self.uart_status()),
            0xC1 | 0xC2 | 0xC4 | 0xC5 => Some(self.io[usize::from(address)]),
            0xC6 => Some(self.datapath.uart.rx_byte()),
            _ => None,
        }
    }

    pub(crate) fn uart_read_register_mut(&mut self, address: u8) -> Option<u8> {
        if address == 0xC6 {
            return Some(self.datapath.uart.read_rx());
        }
        self.uart_read_register(address)
    }

    pub(crate) fn uart_status(&self) -> u8 {
        let uart = &self.datapath.uart;
        let mut value = self.io[0xC0] & 0x03;
        if uart.rx_ready() {
            value |= 0x80;
        }
        if uart.tx_complete() {
            value |= 0x40;
        }
        if uart.tx_ready() {
            value |= 0x20;
        }
        value
    }
}
