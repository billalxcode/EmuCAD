use super::*;

impl Cpu {
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
}

impl Cpu {
    pub(crate) fn service_interrupt(&mut self) -> Result<bool, SimError> {
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

    pub(crate) fn clear_interrupt_flag(&mut self, index: u8) {
        match index {
            14 => self.clear_timer0_compare_flag(),
            16 => self.clear_timer0_overflow_flag(),
            _ => {}
        }
    }

    pub(crate) fn clear_timer0_overflow_flag(&mut self) {
        self.datapath.timers.timer0.take_overflow();
        self.io[0x35] &= !0x01;
    }

    pub(crate) fn clear_timer0_compare_flag(&mut self) {
        self.datapath.timers.timer0.take_compare();
        self.io[0x35] &= !0x02;
    }

    pub(crate) fn finish_sei(&mut self) {
        if self.sei_delay == 0 {
            return;
        }
        self.sei_delay -= 1;
        if self.sei_delay == 0 {
            self.sreg.interrupt = true;
        }
    }

    pub(crate) fn instruction_cycles(&self, mnemonic: Mnemonic, next_pc: u16) -> u8 {
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

    pub(crate) fn wake_ready(&self) -> bool {
        self.sreg.interrupt && self.pending != 0
    }

    pub(crate) fn sleep(&mut self) -> Result<(), SimError> {
        self.sleeping = true;
        Ok(())
    }
}
