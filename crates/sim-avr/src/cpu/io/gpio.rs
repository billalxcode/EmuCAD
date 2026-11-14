use super::*;

impl Cpu {
    pub(crate) fn gpio_read(&self, address: u8) -> Option<u8> {
        Some(match address {
            0x03 => self.datapath.gpio.b.read_pin(&self.circuit),
            0x04 => self.datapath.gpio.b.read_ddr(&self.circuit),
            0x05 => self.datapath.gpio.b.read_port(&self.circuit),
            0x06 => self.datapath.gpio.c.read_pin(&self.circuit),
            0x07 => self.datapath.gpio.c.read_ddr(&self.circuit),
            0x08 => self.datapath.gpio.c.read_port(&self.circuit),
            0x09 => self.datapath.gpio.d.read_pin(&self.circuit),
            0x0A => self.datapath.gpio.d.read_ddr(&self.circuit),
            0x0B => self.datapath.gpio.d.read_port(&self.circuit),
            _ => return None,
        })
    }

    pub(crate) fn gpio_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
        match address {
            0x04 => self.datapath.gpio.b.set_ddr(&mut self.circuit, value)?,
            0x05 => self.datapath.gpio.b.set_port(&mut self.circuit, value)?,
            0x07 => self.datapath.gpio.c.set_ddr(&mut self.circuit, value)?,
            0x08 => self.datapath.gpio.c.set_port(&mut self.circuit, value)?,
            0x0A => self.datapath.gpio.d.set_ddr(&mut self.circuit, value)?,
            0x0B => self.datapath.gpio.d.set_port(&mut self.circuit, value)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn write_data(&mut self, address: u16, value: u8) {
        if (0x20..0x100).contains(&address) {
            let _ = self.io_write(address as u8, value);
            return;
        }
        self.data[usize::from(address) & 0x0FFF] = value;
    }

    pub(crate) fn read_program_byte(&self, address: u16) -> u8 {
        let word = self.program.read(usize::from(address) >> 1).unwrap_or(0);
        if address & 1 == 0 {
            word as u8
        } else {
            (word >> 8) as u8
        }
    }
}
