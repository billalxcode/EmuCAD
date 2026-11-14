use super::*;

impl Cpu {
    pub(super) fn write_register(&mut self, address: u8, value: u8) -> Result<(), SimError> {
        self.set_bits("wa", u64::from(address), 5)?;
        self.set_byte("wd", value)?;
        self.set_bool("we", true)?;
        self.circuit.tick()?;
        self.set_bool("we", false)?;
        self.circuit.settle()
    }

    pub(super) fn set_byte(&mut self, prefix: &str, value: u8) -> Result<(), SimError> {
        self.set_bits(prefix, u64::from(value), 8)
    }

    pub(super) fn set_bits(
        &mut self,
        prefix: &str,
        value: u64,
        width: usize,
    ) -> Result<(), SimError> {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            self.circuit.set_input(&format!("{prefix}{index}"), bit)?;
        }
        Ok(())
    }

    pub(super) fn set_bool(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        self.circuit.set_input(name, value)
    }

    pub(super) fn advance(&mut self, next_pc: u16) -> Result<(), SimError> {
        self.pc = next_pc;
        Ok(())
    }

    pub(super) fn halt(&mut self) -> Result<(), SimError> {
        self.halted = true;
        Ok(())
    }

    pub(super) fn trace(&self, instruction: Instruction, cycles: u8) -> AvrTrace {
        let mut registers = [0u8; 32];
        for (index, slot) in registers.iter_mut().enumerate() {
            *slot = self.register(index as u8);
        }
        AvrTrace {
            pc: self.pc,
            instruction,
            registers,
            sreg: self.sreg,
            halted: self.halted,
            cycles,
            total_cycles: self.cycles,
        }
    }
}
