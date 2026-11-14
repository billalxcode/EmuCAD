use super::*;

impl Cpu {
    pub(crate) fn io_write(&mut self, address: u8, value: u8) -> Result<(), SimError> {
        if self.uart_write_register(address, value) {
            return Ok(());
        }
        if self.timer0_write(address, value)? {
            return Ok(());
        }
        if self.gpio_write(address, value)? {
            return Ok(());
        }
        if address == 0x3F {
            self.sreg = Sreg::from_byte(value);
        } else if let Some(slot) = self.io.get_mut(usize::from(address)) {
            *slot = value;
        }
        Ok(())
    }

    pub(crate) fn uart_write_register(&mut self, address: u8, value: u8) -> bool {
        match address {
            0xC0 => self.write_uart_status(value),
            0xC1 | 0xC2 => self.write_uart_control(address, value),
            0xC4 | 0xC5 => self.write_uart_baud(address, value),
            0xC6 => self.datapath.uart.write_tx(value),
            _ => return false,
        }
        true
    }

    pub(crate) fn write_uart_control(&mut self, address: u8, value: u8) {
        self.io[usize::from(address)] = value;
        self.datapath.uart.write_control(address - 0xC0, value);
    }

    pub(crate) fn write_uart_baud(&mut self, address: u8, value: u8) {
        self.io[usize::from(address)] = value;
        self.sync_uart_baud();
    }

    pub(crate) fn write_uart_status(&mut self, value: u8) {
        if value & 0x40 != 0 {
            self.datapath.uart.take_tx_complete();
        }
        self.io[0xC0] = value & 0x03;
    }

    pub(crate) fn sync_uart_baud(&mut self) {
        let low = self.io[0xC4];
        let high = self.io[0xC5];
        self.datapath.uart.write_baud(low, high);
    }
}
