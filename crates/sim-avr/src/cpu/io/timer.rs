use super::*;

impl Cpu {
    pub(crate) fn timer0_read(&self, address: u8) -> Option<u8> {
        match address {
            0x26 => Some(self.datapath.timers.timer0.counter()),
            0x35 => Some(self.io[0x35]),
            _ => None,
        }
    }

    pub(crate) fn timer0_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
        match address {
            0x24 => self.write_timer0_control(value),
            0x25 => {
                self.io[0x25] = value;
                self.apply_timer0_clock()?;
            }
            0x27 => {
                self.io[0x27] = value;
                self.datapath.timers.timer0.set_compare(value);
            }
            0x28 | 0x2E | 0x6E => self.io[usize::from(address)] = value,
            0x35 => self.write_timer0_flags(value),
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn write_timer0_control(&mut self, value: u8) {
        self.io[0x24] = value;
        let wave = value & 0x03;
        self.datapath.timers.timer0.set_ctc(wave == 0x02);
        if wave == 0x03 {
            self.datapath.timers.timer0.set_pwm(value & 0xC0 == 0xC0);
        }
    }

    pub(crate) fn write_timer0_flags(&mut self, value: u8) {
        if value & 0x01 != 0 {
            self.datapath.timers.timer0.take_overflow();
        }
        if value & 0x02 != 0 {
            self.datapath.timers.timer0.take_compare();
        }
        self.io[0x35] &= !(value & 0x03);
    }

    pub(crate) fn apply_timer0_clock(&mut self) -> Result<(), SimError> {
        let select = match self.io[0x25] & 0x07 {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => return Ok(()),
        };
        self.datapath.timers.timer0.set_prescaler(select)
    }

    pub(crate) fn advance_timer0(&mut self, cycles: u32) -> Result<(), SimError> {
        if !(1..=5).contains(&(self.io[0x25] & 0x07)) {
            return Ok(());
        }
        self.datapath.timers.timer0.tick(cycles)?;
        self.update_timer0_flags()
    }

    pub(crate) fn update_timer0_flags(&mut self) -> Result<(), SimError> {
        let mut flags = self.io[0x35] & 0xF8;
        if self.datapath.timers.timer0.overflow_flag() {
            flags |= 0x01;
        }
        if self.datapath.timers.timer0.compare_flag() {
            flags |= 0x02;
        }
        self.io[0x35] = flags;
        let mask = self.io[0x2E] | self.io[0x6E];
        if flags & 0x01 != 0 && mask & 0x01 != 0 {
            self.request_interrupt(16)?;
        }
        if flags & 0x02 != 0 && mask & 0x02 != 0 {
            self.request_interrupt(14)?;
        }
        Ok(())
    }
}
