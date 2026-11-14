use super::*;

impl Cpu {
    pub(super) fn execute_io(
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

    pub(super) fn set_io_bit(
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

    pub(super) fn execute_in(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let destination = register(instruction.first)?;
        let address = io_address(instruction.second)?;
        let value = self.io_read_mut(address);
        self.write_register(destination, value)
    }

    pub(super) fn execute_out(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let address = io_address(instruction.first)?;
        let source = register(instruction.second)?;
        self.io_write(address, self.register(source))?;
        Ok(())
    }

    pub(super) fn execute_flag(
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

    pub(super) fn set_flag(&mut self, flag: u8, value: bool) {
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

    pub(super) fn execute_bst(&mut self, instruction: &Instruction) -> Result<(), SimError> {
        let value = self.register(register(instruction.first)?);
        let bit = bit(instruction.second)?;
        self.sreg.transfer = (value >> bit) & 1 != 0;
        Ok(())
    }

    pub(super) fn execute_bld(
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

    pub(super) fn execute_memory(
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

    pub(super) fn execute_indirect(
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

    pub(super) fn access_indirect(
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

    pub(super) fn execute_displacement(
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

    pub(super) fn displacement_address(
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

    pub(super) fn execute_direct(
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

    pub(super) fn execute_lpm(
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

    pub(super) fn read_pointer(&self, low: u8) -> u16 {
        u16::from(self.register(low)) | (u16::from(self.register(low + 1)) << 8)
    }

    pub(super) fn write_pointer(&mut self, low: u8, value: u16) -> Result<(), SimError> {
        self.write_register(low, value as u8)?;
        self.write_register(low + 1, (value >> 8) as u8)
    }

    pub(super) fn read_data(&mut self, address: u16) -> u8 {
        if (0x20..0x100).contains(&address) {
            return self.io_read_mut(address as u8);
        }
        self.data[usize::from(address) & 0x0FFF]
    }

    pub(super) fn io_read(&self, address: u8) -> u8 {
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

    pub(super) fn io_read_mut(&mut self, address: u8) -> u8 {
        if let Some(value) = self.uart_read_register_mut(address) {
            return value;
        }
        self.io_read(address)
    }

    pub(super) fn uart_read_register(&self, address: u8) -> Option<u8> {
        match address {
            0xC0 => Some(self.uart_status()),
            0xC1 | 0xC2 | 0xC4 | 0xC5 => Some(self.io[usize::from(address)]),
            0xC6 => Some(self.datapath.uart.rx_byte()),
            _ => None,
        }
    }

    pub(super) fn uart_read_register_mut(&mut self, address: u8) -> Option<u8> {
        if address == 0xC6 {
            return Some(self.datapath.uart.read_rx());
        }
        self.uart_read_register(address)
    }

    pub(super) fn uart_status(&self) -> u8 {
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

    pub(super) fn io_write(&mut self, address: u8, value: u8) -> Result<(), SimError> {
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

    pub(super) fn uart_write_register(&mut self, address: u8, value: u8) -> bool {
        match address {
            0xC0 => self.write_uart_status(value),
            0xC1 | 0xC2 => self.write_uart_control(address, value),
            0xC4 | 0xC5 => self.write_uart_baud(address, value),
            0xC6 => self.datapath.uart.write_tx(value),
            _ => return false,
        }
        true
    }

    pub(super) fn write_uart_control(&mut self, address: u8, value: u8) {
        self.io[usize::from(address)] = value;
        self.datapath.uart.write_control(address - 0xC0, value);
    }

    pub(super) fn write_uart_baud(&mut self, address: u8, value: u8) {
        self.io[usize::from(address)] = value;
        self.sync_uart_baud();
    }

    pub(super) fn write_uart_status(&mut self, value: u8) {
        if value & 0x40 != 0 {
            self.datapath.uart.take_tx_complete();
        }
        self.io[0xC0] = value & 0x03;
    }

    pub(super) fn sync_uart_baud(&mut self) {
        let low = self.io[0xC4];
        let high = self.io[0xC5];
        self.datapath.uart.write_baud(low, high);
    }

    pub(super) fn timer0_read(&self, address: u8) -> Option<u8> {
        match address {
            0x26 => Some(self.datapath.timers.timer0.counter()),
            0x35 => Some(self.io[0x35]),
            _ => None,
        }
    }

    pub(super) fn timer0_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
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

    pub(super) fn write_timer0_control(&mut self, value: u8) {
        self.io[0x24] = value;
        let wave = value & 0x03;
        self.datapath.timers.timer0.set_ctc(wave == 0x02);
        if wave == 0x03 {
            self.datapath.timers.timer0.set_pwm(value & 0xC0 == 0xC0);
        }
    }

    pub(super) fn write_timer0_flags(&mut self, value: u8) {
        if value & 0x01 != 0 {
            self.datapath.timers.timer0.take_overflow();
        }
        if value & 0x02 != 0 {
            self.datapath.timers.timer0.take_compare();
        }
        self.io[0x35] &= !(value & 0x03);
    }

    pub(super) fn apply_timer0_clock(&mut self) -> Result<(), SimError> {
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

    pub(super) fn advance_timer0(&mut self, cycles: u32) -> Result<(), SimError> {
        if !(1..=5).contains(&(self.io[0x25] & 0x07)) {
            return Ok(());
        }
        self.datapath.timers.timer0.tick(cycles)?;
        self.update_timer0_flags()
    }

    pub(super) fn update_timer0_flags(&mut self) -> Result<(), SimError> {
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

    pub(super) fn gpio_read(&self, address: u8) -> Option<u8> {
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

    pub(super) fn gpio_write(&mut self, address: u8, value: u8) -> Result<bool, SimError> {
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

    pub(super) fn write_data(&mut self, address: u16, value: u8) {
        if (0x20..0x100).contains(&address) {
            let _ = self.io_write(address as u8, value);
            return;
        }
        self.data[usize::from(address) & 0x0FFF] = value;
    }

    pub(super) fn read_program_byte(&self, address: u16) -> u8 {
        let word = self.program.read(usize::from(address) >> 1).unwrap_or(0);
        if address & 1 == 0 {
            word as u8
        } else {
            (word >> 8) as u8
        }
    }
}
