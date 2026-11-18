use super::*;

impl Cpu {
    pub(crate) fn execute_memory(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::LddY | Mnemonic::LddZ => {
                self.execute_displacement(instruction, next_pc, true)
            }
            Mnemonic::StdY | Mnemonic::StdZ => {
                self.execute_displacement(instruction, next_pc, false)
            }
            Mnemonic::Lds => self.execute_direct(instruction, next_pc, true),
            Mnemonic::Sts => self.execute_direct(instruction, next_pc, false),
            Mnemonic::LpmZ | Mnemonic::LpmZPost | Mnemonic::ElpmZ | Mnemonic::ElpmZPost => {
                self.execute_lpm(instruction, next_pc)
            }
            _ => self.execute_indirect(instruction, next_pc),
        }
    }

    pub(crate) fn execute_indirect(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mode =
            indirect_mode(instruction.mnemonic).ok_or_else(|| unsupported(instruction.mnemonic))?;
        let mut address = self.read_pointer(mode.base);
        if mode.pre {
            address = address.wrapping_sub(1);
            self.write_pointer(mode.base, address)?;
        }
        self.access_indirect(instruction.first, mode.load, address)?;
        if mode.post {
            address = address.wrapping_add(1);
            self.write_pointer(mode.base, address)?;
        }
        self.advance(next_pc)
    }

    pub(crate) fn access_indirect(
        &mut self,
        operand: Operand,
        load: bool,
        address: u16,
    ) -> Result<(), SimError> {
        if load {
            let destination = register(operand)?;
            let value = self.read_data(address);
            self.write_register(destination, value)
        } else {
            let source = register(operand)?;
            let value = self.register(source);
            self.write_data(address, value);
            Ok(())
        }
    }

    pub(crate) fn execute_displacement(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        load: bool,
    ) -> Result<(), SimError> {
        let base = pointer_base(instruction.mnemonic);
        let address = self.displacement_address(instruction, load, base)?;
        let operand = if load {
            instruction.first
        } else {
            instruction.second
        };
        self.access_indirect(operand, load, address)?;
        self.advance(next_pc)
    }

    pub(crate) fn displacement_address(
        &self,
        instruction: &Instruction,
        load: bool,
        base: u8,
    ) -> Result<u16, SimError> {
        let displacement = if load {
            displacement(instruction.second)?
        } else {
            displacement(instruction.first)?
        };
        Ok(self
            .read_pointer(base)
            .wrapping_add(u16::from(displacement)))
    }

    pub(crate) fn execute_direct(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        load: bool,
    ) -> Result<(), SimError> {
        let address = data_address(instruction.second)?;
        if load {
            let destination = register(instruction.first)?;
            let value = self.read_data(address);
            self.write_register(destination, value)?;
        } else {
            let source = register(instruction.first)?;
            let value = self.register(source);
            self.write_data(address, value);
        }
        self.advance(next_pc)
    }

    pub(crate) fn execute_lpm(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let address = self.read_pointer(30);
        let value = self.read_program_byte(address);
        self.write_register(destination, value)?;
        if matches!(
            instruction.mnemonic,
            Mnemonic::LpmZPost | Mnemonic::ElpmZPost
        ) {
            self.write_pointer(30, address.wrapping_add(1))?;
        }
        self.advance(next_pc)
    }
}
