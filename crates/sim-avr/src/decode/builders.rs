use super::*;

pub(super) fn two_reg(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = register_index(first);
    let r = (((first >> 5) & 0x10) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
}

pub(super) fn immediate(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = 16 + ((first >> 4) & 0xF) as u8;
    let k = (((first & 0x0F00) >> 4) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Immediate(k), 1)
}

pub(super) fn movw(first: u16) -> Instruction {
    let d = (((first >> 4) & 0xF) << 1) as u8;
    let r = ((first & 0xF) << 1) as u8;
    Instruction::new(
        Mnemonic::Movw,
        Operand::Register(d),
        Operand::Register(r),
        1,
    )
}

pub(super) fn muls(first: u16) -> Instruction {
    let d = 16 + ((first >> 4) & 0xF) as u8;
    let r = 16 + (first & 0xF) as u8;
    Instruction::new(
        Mnemonic::Muls,
        Operand::Register(d),
        Operand::Register(r),
        1,
    )
}

pub(super) fn mul_family(first: u16) -> Instruction {
    let mnemonic = match first & 0x0088 {
        0x0000 => Mnemonic::Mulsu,
        0x0008 => Mnemonic::Fmul,
        0x0080 => Mnemonic::Fmuls,
        _ => Mnemonic::Fmulsu,
    };
    let d = 16 + ((first >> 4) & 0x7) as u8;
    let r = 16 + (first & 0x7) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
}

pub(super) fn word_immediate(first: u16, mnemonic: Mnemonic) -> Instruction {
    let d = 24 + ((first >> 3) & 0x6) as u8;
    let k = (((first & 0x00C0) >> 2) | (first & 0xF)) as u8;
    Instruction::new(mnemonic, Operand::Register(d), Operand::Immediate(k), 1)
}

pub(super) fn io_bit(first: u16, mnemonic: Mnemonic) -> Instruction {
    let io = ((first >> 3) & 0x1F) as u8;
    let bit = (first & 0x7) as u8;
    Instruction::new(mnemonic, Operand::Io(io), Operand::Bit(bit), 1)
}

pub(super) fn branch(first: u16) -> Instruction {
    let mnemonic = if first & 0x0400 == 0 {
        Mnemonic::Brbs
    } else {
        Mnemonic::Brbc
    };
    let flag = (first & 0x7) as u8;
    let offset = sign_extend((first >> 3) & 0x7F, 7);
    Instruction::new(mnemonic, Operand::Flag(flag), Operand::Offset(offset), 1)
}

pub(super) fn bit_register(first: u16, mnemonic: Mnemonic) -> Instruction {
    Instruction::new(
        mnemonic,
        Operand::Register(register_index(first)),
        Operand::Bit((first & 0x7) as u8),
        1,
    )
}

pub(super) fn skip_bit(first: u16) -> Instruction {
    let mnemonic = if first & 0x0200 != 0 {
        Mnemonic::Sbrs
    } else {
        Mnemonic::Sbrc
    };
    bit_register(first, mnemonic)
}

pub(super) fn sreg_bit(first: u16) -> Instruction {
    let mnemonic = if first & 0x0080 == 0 {
        Mnemonic::Bset
    } else {
        Mnemonic::Bclr
    };
    Instruction::new(
        mnemonic,
        Operand::Flag(((first >> 4) & 0x7) as u8),
        Operand::None,
        1,
    )
}

pub(super) fn relative(first: u16, mnemonic: Mnemonic) -> Instruction {
    let offset = sign_extend(first & 0x0FFF, 12);
    Instruction::new(mnemonic, Operand::Offset(offset), Operand::None, 1)
}

pub(super) fn lds(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    let address = second.ok_or_else(|| truncated(first))?;
    Ok(Instruction::new(
        Mnemonic::Lds,
        Operand::Register(register_index(first)),
        Operand::Address(address),
        2,
    ))
}

pub(super) fn sts(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    let address = second.ok_or_else(|| truncated(first))?;
    Ok(Instruction::new(
        Mnemonic::Sts,
        Operand::Register(register_index(first)),
        Operand::Address(address),
        2,
    ))
}

pub(super) fn register_instruction(first: u16, mnemonic: Mnemonic) -> Instruction {
    Instruction::new(
        mnemonic,
        Operand::Register(register_index(first)),
        Operand::None,
        1,
    )
}

pub(super) fn lpm_fixed(mnemonic: Mnemonic) -> Instruction {
    Instruction::new(mnemonic, Operand::Register(0), Operand::None, 1)
}

pub(super) fn fixed(mnemonic: Mnemonic) -> Instruction {
    Instruction::new(mnemonic, Operand::None, Operand::None, 1)
}
