use super::*;

impl Cpu {
    pub(super) fn write_register(&mut self, address: u8, value: u8) -> Result<(), SimError> {
        if let Some(slot) = self.registers.get_mut(usize::from(address)) {
            *slot = value;
        }
        Ok(())
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
        AvrTrace {
            pc: self.pc,
            instruction,
            registers: self.registers,
            sreg: self.sreg,
            halted: self.halted,
            cycles,
            total_cycles: self.cycles,
        }
    }
}
