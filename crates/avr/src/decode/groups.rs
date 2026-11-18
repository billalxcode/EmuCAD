use super::*;

pub(super) fn decode_group_0(first: u16) -> Result<Instruction, SimError> {
    if first == 0 {
        return Ok(fixed(Mnemonic::Nop));
    }
    match first & 0xFC00 {
        0x0400 => Ok(two_reg(first, Mnemonic::Cpc)),
        0x0800 => Ok(two_reg(first, Mnemonic::Sbc)),
        0x0C00 => Ok(two_reg(first, Mnemonic::Add)),
        0x0000 => decode_group_0_misc(first),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_group_0_misc(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFF00 {
        0x0100 => Ok(movw(first)),
        0x0200 => Ok(muls(first)),
        0x0300 => Ok(mul_family(first)),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_group_1(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFC00 {
        0x1000 => Ok(two_reg(first, Mnemonic::Cpse)),
        0x1400 => Ok(two_reg(first, Mnemonic::Cp)),
        0x1800 => Ok(two_reg(first, Mnemonic::Sub)),
        0x1C00 => Ok(two_reg(first, Mnemonic::Adc)),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_group_2(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFC00 {
        0x2000 => Ok(two_reg(first, Mnemonic::And)),
        0x2400 => Ok(two_reg(first, Mnemonic::Eor)),
        0x2800 => Ok(two_reg(first, Mnemonic::Or)),
        0x2C00 => Ok(two_reg(first, Mnemonic::Mov)),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_ldd(first: u16) -> Result<Instruction, SimError> {
    let store = first & 0x0200 != 0;
    let y = first & 0x0008 != 0;
    let mnemonic = match (store, y) {
        (false, false) => Mnemonic::LddZ,
        (false, true) => Mnemonic::LddY,
        (true, false) => Mnemonic::StdZ,
        (true, true) => Mnemonic::StdY,
    };
    let d = register_index(first);
    let q = (((first & 0x2000) >> 8) | ((first & 0x0C00) >> 7) | (first & 0x7)) as u8;
    Ok(if store {
        Instruction::new(mnemonic, Operand::Displacement(q), Operand::Register(d), 1)
    } else {
        Instruction::new(mnemonic, Operand::Register(d), Operand::Displacement(q), 1)
    })
}

pub(super) fn decode_group_9(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    if first & 0xFF0F == 0x9408 {
        return Ok(sreg_bit(first));
    }
    if let Some(instruction) = fixed_9(first) {
        return Ok(instruction);
    }
    match first & 0xFE0F {
        0x9000 => lds(first, second),
        0x9200 => sts(first, second),
        _ => decode_9_masked(first, second),
    }
}

pub(super) fn fixed_9(first: u16) -> Option<Instruction> {
    let mnemonic = match first {
        0x9409 => Mnemonic::Ijmp,
        0x9419 => Mnemonic::Eijmp,
        0x9509 => Mnemonic::Icall,
        0x9519 => Mnemonic::Eicall,
        0x9508 => Mnemonic::Ret,
        0x9518 => Mnemonic::Reti,
        0x9588 => Mnemonic::Sleep,
        0x9598 => Mnemonic::Break,
        0x95A8 => Mnemonic::Wdr,
        0x95E8 => Mnemonic::Spm,
        0x95C8 => return Some(lpm_fixed(Mnemonic::LpmZ)),
        0x95D8 => return Some(lpm_fixed(Mnemonic::ElpmZ)),
        _ => return None,
    };
    Some(fixed(mnemonic))
}

pub(super) fn decode_9_masked(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    if let Some(instruction) = decode_9_memory(first) {
        return Ok(instruction);
    }
    if let Some(instruction) = decode_9_alu(first, second)? {
        return Ok(instruction);
    }
    decode_9_io(first)
}

pub(super) fn decode_9_memory(first: u16) -> Option<Instruction> {
    if let Some(instruction) = load_memory(first) {
        return Some(instruction);
    }
    store_memory(first)
}

pub(super) fn load_memory(first: u16) -> Option<Instruction> {
    let mnemonic = match first & 0xFE0F {
        0x9001 => Mnemonic::LdZPost,
        0x9002 => Mnemonic::LdZPre,
        0x9004 => Mnemonic::LpmZ,
        0x9005 => Mnemonic::LpmZPost,
        0x9006 => Mnemonic::ElpmZ,
        0x9007 => Mnemonic::ElpmZPost,
        0x9009 => Mnemonic::LdYPost,
        0x900A => Mnemonic::LdYPre,
        0x900C => Mnemonic::LdX,
        0x900D => Mnemonic::LdXPost,
        0x900E => Mnemonic::LdXPre,
        0x900F => Mnemonic::Pop,
        _ => return None,
    };
    Some(register_instruction(first, mnemonic))
}

pub(super) fn store_memory(first: u16) -> Option<Instruction> {
    let mnemonic = match first & 0xFE0F {
        0x9201 => Mnemonic::StZPost,
        0x9202 => Mnemonic::StZPre,
        0x9209 => Mnemonic::StYPost,
        0x920A => Mnemonic::StYPre,
        0x920C => Mnemonic::StX,
        0x920D => Mnemonic::StXPost,
        0x920E => Mnemonic::StXPre,
        0x920F => Mnemonic::Push,
        _ => return None,
    };
    Some(register_instruction(first, mnemonic))
}

pub(super) fn decode_9_alu(
    first: u16,
    second: Option<u16>,
) -> Result<Option<Instruction>, SimError> {
    let mnemonic = match first & 0xFE0F {
        0x9400 => Mnemonic::Com,
        0x9401 => Mnemonic::Neg,
        0x9402 => Mnemonic::Swap,
        0x9403 => Mnemonic::Inc,
        0x9405 => Mnemonic::Asr,
        0x9406 => Mnemonic::Lsr,
        0x9407 => Mnemonic::Ror,
        0x940A => Mnemonic::Dec,
        _ => return decode_9_long(first, second),
    };
    Ok(Some(register_instruction(first, mnemonic)))
}

pub(super) fn decode_9_long(
    first: u16,
    second: Option<u16>,
) -> Result<Option<Instruction>, SimError> {
    let mnemonic = match first & 0xFE0E {
        0x940C => Mnemonic::Jmp,
        0x940E => Mnemonic::Call,
        _ => return Ok(None),
    };
    let next = second.ok_or_else(|| truncated(first))?;
    let high = (((first & 0x01F0) >> 3) | (first & 1)) as u32;
    let address = (high << 16) | u32::from(next);
    Ok(Some(Instruction::new(
        mnemonic,
        Operand::LongAddress(address),
        Operand::None,
        2,
    )))
}

pub(super) fn decode_9_io(first: u16) -> Result<Instruction, SimError> {
    if first & 0xFC00 == 0x9C00 {
        return Ok(two_reg(first, Mnemonic::Mul));
    }
    match first & 0xFF00 {
        0x9600 => Ok(word_immediate(first, Mnemonic::Adiw)),
        0x9700 => Ok(word_immediate(first, Mnemonic::Sbiw)),
        0x9800 => Ok(io_bit(first, Mnemonic::Cbi)),
        0x9900 => Ok(io_bit(first, Mnemonic::Sbic)),
        0x9A00 => Ok(io_bit(first, Mnemonic::Sbi)),
        0x9B00 => Ok(io_bit(first, Mnemonic::Sbis)),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_group_f(first: u16) -> Result<Instruction, SimError> {
    match first & 0xFE00 {
        0xF000 | 0xF200 | 0xF400 | 0xF600 => Ok(branch(first)),
        0xF800 | 0xF900 => Ok(bit_register(first, Mnemonic::Bld)),
        0xFA00 | 0xFB00 => Ok(bit_register(first, Mnemonic::Bst)),
        0xFC00 | 0xFE00 => Ok(skip_bit(first)),
        _ => Err(illegal(first)),
    }
}

pub(super) fn decode_in_out(first: u16) -> Result<Instruction, SimError> {
    let io = ((((first >> 9) & 0x3) << 4) | (first & 0xF)) as u8;
    let d = register_index(first);
    match first & 0xF800 {
        0xB000 => Ok(Instruction::new(
            Mnemonic::In,
            Operand::Register(d),
            Operand::Io(io),
            1,
        )),
        0xB800 => Ok(Instruction::new(
            Mnemonic::Out,
            Operand::Io(io),
            Operand::Register(d),
            1,
        )),
        _ => Err(illegal(first)),
    }
}
