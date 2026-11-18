use super::*;

impl Cpu {
    pub(crate) fn execute_io(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Sbi => self.set_io_bit(instruction, true)?,
            Mnemonic::Cbi => self.set_io_bit(instruction, false)?,
            Mnemonic::In => self.execute_in(instruction)?,
            Mnemonic::Out => self.execute_out(instruction)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    pub(crate) fn set_io_bit(
        &mut self,
        instruction: &Instruction,
        set: bool,
    ) -> Result<(), SimError> {
        let address = io_address(instruction.first)?;
        let bit = bit(instruction.second)?;
        let mut value = self.io_read(address);
        if set {
            value |= 1 << bit;
        } else {
            value &= !(1 << bit);
        }
        self.io_write(address, value)?;
        Ok(())
    }

    pub(crate) fn execute_in(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let address = io_address(instruction.second)?;
        let value = self.io_read_mut(address);
        self.write_register(destination, value)
    }

    pub(crate) fn execute_out(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let address = io_address(instruction.first)?;
        let source = register(instruction.second)?;
        self.io_write(address, self.register(source))?;
        Ok(())
    }

    pub(crate) fn execute_flag(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Bset => self.set_flag(flag(instruction.first)?, true),
            Mnemonic::Bclr => self.set_flag(flag(instruction.first)?, false),
            Mnemonic::Bst => self.execute_bst(instruction)?,
            Mnemonic::Bld => return self.execute_bld(instruction, next_pc),
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    pub(crate) fn set_flag(&mut self, flag: u8, value: bool) {
        if flag == 7 {
            if value {
                self.sei_delay = 2;
            } else {
                self.sreg.interrupt = false;
                self.sei_delay = 0;
            }
        } else {
            self.sreg.set(flag, value);
        }
    }

    pub(crate) fn execute_bst(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let value = self.register(register(instruction.first)?);
        let bit = bit(instruction.second)?;
        self.sreg.transfer = (value >> bit) & 1 != 0;
        Ok(())
    }

    pub(crate) fn execute_bld(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let bit = bit(instruction.second)?;
        let mut value = self.register(destination);
        if self.sreg.transfer {
            value |= 1 << bit;
        } else {
            value &= !(1 << bit);
        }
        self.write_register(destination, value)?;
        self.advance(next_pc)
    }
}
