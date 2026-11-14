use super::*;

impl Cpu {
    pub(super) fn execute(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Nop | Mnemonic::Wdr => return self.advance(next_pc),
            Mnemonic::Break => return self.halt(),
            Mnemonic::Sleep => return self.sleep(),
            _ => {}
        }
        self.execute_complex(instruction, next_pc)
    }

    pub(super) fn execute_complex(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_move(mnemonic) {
            return self.execute_move(instruction, next_pc);
        }
        if is_arithmetic(mnemonic) {
            return self.execute_arithmetic(instruction, next_pc);
        }
        if is_logic(mnemonic) {
            return self.execute_logic(instruction, next_pc);
        }
        self.execute_special(instruction, next_pc)
    }

    pub(super) fn execute_special(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_shift(mnemonic) {
            return self.execute_shift(instruction, next_pc);
        }
        if mnemonic == Mnemonic::Mul {
            return self.execute_multiply(instruction, next_pc);
        }
        self.execute_control(instruction, next_pc)
    }

    pub(super) fn execute_control(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_flow(mnemonic) {
            return self.execute_flow(instruction, next_pc);
        }
        if is_stack(mnemonic) {
            return self.execute_stack(instruction, next_pc);
        }
        if is_call(mnemonic) {
            return self.execute_call(instruction, next_pc);
        }
        self.execute_io_flag(instruction, next_pc)
    }

    pub(super) fn execute_stack(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Push => {
                let source = register(instruction.first)?;
                self.push_byte(self.register(source));
            }
            Mnemonic::Pop => {
                let destination = register(instruction.first)?;
                let value = self.pop_byte();
                self.write_register(destination, value)?;
            }
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    pub(super) fn execute_call(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Rcall => self.call_relative(instruction, next_pc),
            Mnemonic::Call => self.call_absolute(instruction, next_pc),
            Mnemonic::Icall | Mnemonic::Eicall => self.call_indirect(next_pc),
            Mnemonic::Ijmp | Mnemonic::Eijmp => {
                self.pc = self.read_pointer(30);
                Ok(())
            }
            Mnemonic::Ret => self.return_from(false),
            Mnemonic::Reti => self.return_from(true),
            _ => Err(unsupported(instruction.mnemonic)),
        }
    }

    pub(super) fn return_from(&mut self, interrupt: bool) -> Result<(), SimError> {
        self.pc = self.pop_word();
        if interrupt {
            self.sreg.interrupt = true;
        }
        Ok(())
    }

    pub(super) fn call_relative(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let offset = offset(instruction.first)?;
        self.push_word(next_pc);
        self.pc = next_pc.wrapping_add(offset as u16);
        Ok(())
    }

    pub(super) fn call_absolute(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let address = long_address(instruction.first)? as u16;
        self.push_word(next_pc);
        self.pc = address;
        Ok(())
    }

    pub(super) fn call_indirect(&mut self, next_pc: u16) -> Result<(), SimError> {
        let address = self.read_pointer(30);
        self.push_word(next_pc);
        self.pc = address;
        Ok(())
    }

    pub(super) fn push_byte(&mut self, value: u8) {
        self.write_data(self.sp, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub(super) fn pop_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read_data(self.sp)
    }

    pub(super) fn push_word(&mut self, value: u16) {
        self.push_byte(value as u8);
        self.push_byte((value >> 8) as u8);
    }

    pub(super) fn pop_word(&mut self) -> u16 {
        let high = self.pop_byte();
        let low = self.pop_byte();
        u16::from(low) | (u16::from(high) << 8)
    }

    pub(super) fn execute_io_flag(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let mnemonic = instruction.mnemonic;
        if is_memory(mnemonic) {
            return self.execute_memory(instruction, next_pc);
        }
        if is_skip(mnemonic) {
            return self.execute_skip(instruction, next_pc);
        }
        if is_io(mnemonic) {
            return self.execute_io(instruction, next_pc);
        }
        if is_flag(mnemonic) {
            return self.execute_flag(instruction, next_pc);
        }
        Err(unsupported(mnemonic))
    }

    pub(super) fn execute_move(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Ldi => self.load_immediate(instruction)?,
            Mnemonic::Mov => self.move_register(instruction)?,
            Mnemonic::Movw => self.execute_movw(instruction)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        self.advance(next_pc)
    }

    fn load_immediate(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let value = immediate(instruction.second)?;
        self.write_register(destination, value)
    }

    fn move_register(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let source = register(instruction.second)?;
        self.write_register(destination, self.register(source))
    }

    pub(super) fn execute_movw(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let source = register(instruction.second)?;
        let low = self.register(source);
        let high = self.register(source + 1);
        self.write_register(destination, low)?;
        self.write_register(destination + 1, high)
    }

    pub(super) fn execute_arithmetic(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        if matches!(instruction.mnemonic, Mnemonic::Adiw | Mnemonic::Sbiw) {
            return self.execute_word_immediate(instruction, next_pc);
        }
        if let Some((destination, value)) = update_operand(instruction)? {
            return self.execute_update(instruction.mnemonic, destination, value, next_pc);
        }
        self.execute_binary_arithmetic(instruction, next_pc)
    }

    pub(super) fn execute_word_immediate(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let immediate_value = u16::from(immediate(instruction.second)?);
        let value = self.read_pointer(destination);
        let (result, carry, overflow) =
            word_arithmetic(instruction.mnemonic, value, immediate_value);
        self.write_pointer(destination, result)?;
        self.sreg.carry = carry;
        self.sreg.zero = result == 0;
        self.sreg.negative = result & 0x8000 != 0;
        self.sreg.overflow = overflow;
        self.sreg.sign = self.sreg.negative ^ self.sreg.overflow;
        self.advance(next_pc)
    }

    pub(super) fn execute_binary_arithmetic(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let (source, immediate_value) = second_operand(instruction.second)?;
        let (subtract, use_immediate, carry, write) = arithmetic_control(instruction.mnemonic)?;
        self.load_arithmetic(destination, source, immediate_value, use_immediate)?;
        self.set_bool("sub", subtract)?;
        self.set_bool("cin", carry && self.sreg.carry)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.arithmetic.result) as u8;
        self.update_binary_flags(instruction.mnemonic);
        if write {
            self.write_register(destination, result)?;
        }
        self.advance(next_pc)
    }

    pub(super) fn update_binary_flags(&mut self, mnemonic: Mnemonic) {
        let previous_zero = self.sreg.zero;
        self.update_full_flags();
        if has_cumulative_zero(mnemonic) {
            self.sreg.zero &= previous_zero;
        }
    }

    pub(super) fn execute_update(
        &mut self,
        mnemonic: Mnemonic,
        destination: u8,
        value: u8,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let (first, second, subtract) = update_values(mnemonic, self.register(destination), value);
        self.set_byte("a", first)?;
        self.set_byte("b", second)?;
        self.set_bool("imm", false)?;
        self.set_bool("sub", subtract)?;
        self.set_bool("cin", false)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.arithmetic.result) as u8;
        self.update_partial_flags();
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    pub(super) fn execute_logic(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let (op, use_immediate) = logic_control(instruction.mnemonic)?;
        let destination = register(instruction.first)?;
        let (source, immediate_value) = if instruction.mnemonic == Mnemonic::Com {
            (0, 0)
        } else {
            second_operand(instruction.second)?
        };
        self.load_arithmetic(destination, source, immediate_value, use_immediate)?;
        self.set_bits("lop", u64::from(op), 2)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.logic.result) as u8;
        self.update_logic_flags(instruction.mnemonic);
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    pub(super) fn execute_shift(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let op = shift_op(instruction.mnemonic);
        self.set_byte("a", self.register(destination))?;
        self.set_bool("cin", self.sreg.carry)?;
        self.set_bits("sop", u64::from(op), 2)?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.shift.result) as u8;
        self.update_shift_flags();
        self.write_register(destination, result)?;
        self.advance(next_pc)
    }

    pub(super) fn execute_multiply(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        let first = register(instruction.first)?;
        let second = register(instruction.second)?;
        self.set_byte("a", self.register(first))?;
        self.set_byte("b", self.register(second))?;
        self.circuit.settle()?;
        let result = read_bits(&self.circuit, &self.datapath.multiply.result);
        self.sreg.zero = self.circuit.value(self.datapath.multiply.zero);
        self.sreg.carry = self.circuit.value(self.datapath.multiply.carry);
        self.write_register(0, result as u8)?;
        self.write_register(1, (result >> 8) as u8)?;
        self.advance(next_pc)
    }

    pub(super) fn execute_flow(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        match instruction.mnemonic {
            Mnemonic::Rjmp => {
                let offset = offset(instruction.first)?;
                self.pc = next_pc.wrapping_add(offset as u16);
            }
            Mnemonic::Jmp => self.pc = long_address(instruction.first)? as u16,
            Mnemonic::Brbs => self.branch(instruction, next_pc, true)?,
            Mnemonic::Brbc => self.branch(instruction, next_pc, false)?,
            _ => return Err(unsupported(instruction.mnemonic)),
        }
        Ok(())
    }

    pub(super) fn branch(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
        set: bool,
    ) -> Result<(), SimError> {
        let flag = flag(instruction.first)?;
        let offset = offset(instruction.second)?;
        if self.sreg.get(flag) == set {
            self.pc = next_pc.wrapping_add(offset as u16);
        } else {
            self.pc = next_pc;
        }
        Ok(())
    }

    pub(super) fn execute_skip(
        &mut self,
        instruction: &Instruction,
        next_pc: u16,
    ) -> Result<(), SimError> {
        if self.skip_condition(instruction)? {
            self.pc = self.skipped_pc(next_pc)?;
        } else {
            self.pc = next_pc;
        }
        Ok(())
    }

    pub(super) fn skip_condition(&self, instruction: &Instruction) -> Result<bool, SimError> {
        match instruction.mnemonic {
            Mnemonic::Cpse => Ok(self.register(register(instruction.first)?)
                == self.register(register(instruction.second)?)),
            Mnemonic::Sbrc => Ok(self.bit_value(instruction)? == 0),
            Mnemonic::Sbrs => Ok(self.bit_value(instruction)? != 0),
            Mnemonic::Sbic => Ok(self.io_bit(instruction)? == 0),
            Mnemonic::Sbis => Ok(self.io_bit(instruction)? != 0),
            _ => Err(unsupported(instruction.mnemonic)),
        }
    }

    pub(super) fn bit_value(&self, instruction: &Instruction) -> Result<u8, SimError> {
        let value = self.register(register(instruction.first)?);
        let bit = bit(instruction.second)?;
        Ok((value >> bit) & 1)
    }

    pub(super) fn io_bit(&self, instruction: &Instruction) -> Result<u8, SimError> {
        let value = self.io(io_address(instruction.first)?);
        let bit = bit(instruction.second)?;
        Ok((value >> bit) & 1)
    }

    pub(super) fn skipped_pc(&self, next_pc: u16) -> Result<u16, SimError> {
        let fetched = self.program.fetch(next_pc)?;
        Ok(next_pc.wrapping_add(u16::from(fetched.words)))
    }
}
