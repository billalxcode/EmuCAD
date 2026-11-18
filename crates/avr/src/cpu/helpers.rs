use super::*;

pub(super) fn is_move(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Ldi | Mnemonic::Mov | Mnemonic::Movw)
}

pub(super) fn word_arithmetic(mnemonic: Mnemonic, value: u16, immediate: u16) -> (u16, bool, bool) {
    if mnemonic == Mnemonic::Adiw {
        let result = value.wrapping_add(immediate);
        let carry = value & 0x8000 != 0 && result & 0x8000 == 0;
        let overflow = value & 0x8000 == 0 && result & 0x8000 != 0;
        return (result, carry, overflow);
    }
    let result = value.wrapping_sub(immediate);
    let carry = value & 0x8000 == 0 && result & 0x8000 != 0;
    let overflow = value & 0x8000 != 0 && result & 0x8000 == 0;
    (result, carry, overflow)
}

pub(super) fn is_arithmetic(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Add
            | Mnemonic::Adc
            | Mnemonic::Sub
            | Mnemonic::Sbc
            | Mnemonic::Subi
            | Mnemonic::Sbci
            | Mnemonic::Cp
            | Mnemonic::Cpc
            | Mnemonic::Cpi
            | Mnemonic::Neg
            | Mnemonic::Inc
            | Mnemonic::Dec
            | Mnemonic::Adiw
            | Mnemonic::Sbiw
    )
}

pub(super) fn is_multiply(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Mul
            | Mnemonic::Muls
            | Mnemonic::Mulsu
            | Mnemonic::Fmul
            | Mnemonic::Fmuls
            | Mnemonic::Fmulsu
    )
}

pub(super) fn is_logic(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::And
            | Mnemonic::Andi
            | Mnemonic::Or
            | Mnemonic::Ori
            | Mnemonic::Eor
            | Mnemonic::Com
    )
}

pub(super) fn is_shift(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Lsr | Mnemonic::Asr | Mnemonic::Ror | Mnemonic::Swap
    )
}

pub(super) fn is_flow(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Rjmp | Mnemonic::Jmp | Mnemonic::Brbs | Mnemonic::Brbc
    )
}

pub(super) fn is_stack(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Push | Mnemonic::Pop)
}

pub(super) fn is_call(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Rcall
            | Mnemonic::Call
            | Mnemonic::Icall
            | Mnemonic::Eicall
            | Mnemonic::Ijmp
            | Mnemonic::Eijmp
            | Mnemonic::Ret
            | Mnemonic::Reti
    )
}

pub(super) fn base_cycles(mnemonic: Mnemonic) -> u8 {
    if is_four_cycle(mnemonic) {
        4
    } else if is_three_cycle(mnemonic) {
        3
    } else if is_two_cycle(mnemonic) {
        2
    } else {
        1
    }
}

pub(super) fn is_two_cycle(mnemonic: Mnemonic) -> bool {
    if is_multiply(mnemonic) {
        return true;
    }
    matches!(
        mnemonic,
        Mnemonic::Push
            | Mnemonic::Pop
            | Mnemonic::Rjmp
            | Mnemonic::Ijmp
            | Mnemonic::Eijmp
            | Mnemonic::Adiw
            | Mnemonic::Sbiw
            | Mnemonic::Sbi
            | Mnemonic::Cbi
    ) || is_indirect_load(mnemonic)
        || matches!(mnemonic, Mnemonic::LddY | Mnemonic::LddZ | Mnemonic::Lds)
        || is_store(mnemonic)
}

pub(super) fn is_three_cycle(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Jmp
            | Mnemonic::Rcall
            | Mnemonic::Icall
            | Mnemonic::LpmZ
            | Mnemonic::LpmZPost
            | Mnemonic::ElpmZ
            | Mnemonic::ElpmZPost
    )
}

pub(super) fn is_four_cycle(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Call | Mnemonic::Ret | Mnemonic::Reti | Mnemonic::Eicall
    )
}

pub(super) fn is_skip(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Cpse | Mnemonic::Sbrc | Mnemonic::Sbrs | Mnemonic::Sbic | Mnemonic::Sbis
    )
}

pub(super) fn is_io(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Sbi | Mnemonic::Cbi | Mnemonic::In | Mnemonic::Out
    )
}

pub(super) fn is_flag(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::Bset | Mnemonic::Bclr | Mnemonic::Bst | Mnemonic::Bld
    )
}

pub(super) fn is_memory(mnemonic: Mnemonic) -> bool {
    is_load(mnemonic) || is_store(mnemonic)
}

pub(super) fn is_load(mnemonic: Mnemonic) -> bool {
    is_indirect_load(mnemonic)
        || matches!(
            mnemonic,
            Mnemonic::LddY
                | Mnemonic::LddZ
                | Mnemonic::Lds
                | Mnemonic::LpmZ
                | Mnemonic::LpmZPost
                | Mnemonic::ElpmZ
                | Mnemonic::ElpmZPost
        )
}

pub(super) fn is_indirect_load(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::LdX
            | Mnemonic::LdXPost
            | Mnemonic::LdXPre
            | Mnemonic::LdY
            | Mnemonic::LdYPost
            | Mnemonic::LdYPre
            | Mnemonic::LdZ
            | Mnemonic::LdZPost
            | Mnemonic::LdZPre
    )
}

pub(super) fn is_store(mnemonic: Mnemonic) -> bool {
    matches!(
        mnemonic,
        Mnemonic::StX
            | Mnemonic::StXPost
            | Mnemonic::StXPre
            | Mnemonic::StY
            | Mnemonic::StYPost
            | Mnemonic::StYPre
            | Mnemonic::StZ
            | Mnemonic::StZPost
            | Mnemonic::StZPre
            | Mnemonic::StdY
            | Mnemonic::StdZ
            | Mnemonic::Sts
    )
}

pub(super) fn indirect_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    load_mode(mnemonic).or_else(|| store_mode(mnemonic))
}

pub(super) fn load_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    let (base, post, pre) = match mnemonic {
        Mnemonic::LdX => (26, false, false),
        Mnemonic::LdXPost => (26, true, false),
        Mnemonic::LdXPre => (26, false, true),
        Mnemonic::LdY => (28, false, false),
        Mnemonic::LdYPost => (28, true, false),
        Mnemonic::LdYPre => (28, false, true),
        Mnemonic::LdZ => (30, false, false),
        Mnemonic::LdZPost => (30, true, false),
        Mnemonic::LdZPre => (30, false, true),
        _ => return None,
    };
    Some(MemoryMode {
        base,
        post,
        pre,
        load: true,
    })
}

pub(super) fn store_mode(mnemonic: Mnemonic) -> Option<MemoryMode> {
    let (base, post, pre) = match mnemonic {
        Mnemonic::StX => (26, false, false),
        Mnemonic::StXPost => (26, true, false),
        Mnemonic::StXPre => (26, false, true),
        Mnemonic::StY => (28, false, false),
        Mnemonic::StYPost => (28, true, false),
        Mnemonic::StYPre => (28, false, true),
        Mnemonic::StZ => (30, false, false),
        Mnemonic::StZPost => (30, true, false),
        Mnemonic::StZPre => (30, false, true),
        _ => return None,
    };
    Some(MemoryMode {
        base,
        post,
        pre,
        load: false,
    })
}

pub(super) fn has_cumulative_zero(mnemonic: Mnemonic) -> bool {
    matches!(mnemonic, Mnemonic::Sbc | Mnemonic::Sbci | Mnemonic::Cpc)
}

pub(super) fn arithmetic_control(mnemonic: Mnemonic) -> Result<(bool, bool, bool, bool), SimError> {
    match mnemonic {
        Mnemonic::Add => Ok((false, false, false, true)),
        Mnemonic::Adc => Ok((false, false, true, true)),
        Mnemonic::Sub => Ok((true, false, false, true)),
        Mnemonic::Sbc => Ok((true, false, true, true)),
        Mnemonic::Subi => Ok((true, true, false, true)),
        Mnemonic::Sbci => Ok((true, true, true, true)),
        Mnemonic::Cp => Ok((true, false, false, false)),
        Mnemonic::Cpc => Ok((true, false, true, false)),
        Mnemonic::Cpi => Ok((true, true, false, false)),
        _ => Err(unsupported(mnemonic)),
    }
}

pub(super) fn update_operand(instruction: &Instruction) -> Result<Option<(u8, u8)>, SimError> {
    let destination = register(instruction.first)?;
    Ok(match instruction.mnemonic {
        Mnemonic::Neg => Some((destination, 0)),
        Mnemonic::Inc | Mnemonic::Dec => Some((destination, 1)),
        _ => None,
    })
}

pub(super) fn second_operand(operand: Operand) -> Result<(u8, u8), SimError> {
    match operand {
        Operand::Register(value) => Ok((value, 0)),
        Operand::Immediate(value) => Ok((0, value)),
        _ => Err(SimError::Parse(
            "expected register or immediate".to_string(),
        )),
    }
}

pub(super) fn register(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Register(value) => Ok(value),
        _ => Err(SimError::Parse("expected register".to_string())),
    }
}

pub(super) fn immediate(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Immediate(value) => Ok(value),
        _ => Err(SimError::Parse("expected immediate".to_string())),
    }
}

pub(super) fn flag(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Flag(value) => Ok(value),
        _ => Err(SimError::Parse("expected flag".to_string())),
    }
}

pub(super) fn bit(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Bit(value) => Ok(value),
        _ => Err(SimError::Parse("expected bit index".to_string())),
    }
}

pub(super) fn io_address(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Io(value) => Ok(value),
        _ => Err(SimError::Parse("expected I/O address".to_string())),
    }
}

pub(super) fn displacement(operand: Operand) -> Result<u8, SimError> {
    match operand {
        Operand::Displacement(value) => Ok(value),
        _ => Err(SimError::Parse("expected displacement".to_string())),
    }
}

pub(super) fn data_address(operand: Operand) -> Result<u16, SimError> {
    match operand {
        Operand::Address(value) => Ok(value),
        _ => Err(SimError::Parse("expected data address".to_string())),
    }
}

pub(super) fn pointer_base(mnemonic: Mnemonic) -> u8 {
    if matches!(mnemonic, Mnemonic::LddY | Mnemonic::StdY) {
        28
    } else {
        30
    }
}

pub(super) fn offset(operand: Operand) -> Result<i16, SimError> {
    match operand {
        Operand::Offset(value) => Ok(value),
        _ => Err(SimError::Parse("expected offset".to_string())),
    }
}

pub(super) fn long_address(operand: Operand) -> Result<u32, SimError> {
    match operand {
        Operand::LongAddress(value) => Ok(value),
        _ => Err(SimError::Parse("expected address".to_string())),
    }
}

pub(super) fn unsupported(mnemonic: Mnemonic) -> SimError {
    SimError::Parse(format!("unsupported AVR instruction: {mnemonic:?}"))
}

pub(super) fn logic_value(mnemonic: Mnemonic, first: u8, second: u8) -> Result<u8, SimError> {
    match mnemonic {
        Mnemonic::And | Mnemonic::Andi => Ok(first & second),
        Mnemonic::Or | Mnemonic::Ori => Ok(first | second),
        Mnemonic::Eor => Ok(first ^ second),
        Mnemonic::Com => Ok(!first),
        _ => Err(unsupported(mnemonic)),
    }
}

pub(super) fn shift_value(
    mnemonic: Mnemonic,
    value: u8,
    carry: bool,
) -> Result<(u8, bool), SimError> {
    let carry_out = value & 0x01 != 0;
    match mnemonic {
        Mnemonic::Lsr => Ok((value >> 1, carry_out)),
        Mnemonic::Asr => Ok(((value >> 1) | (value & 0x80), carry_out)),
        Mnemonic::Ror => Ok(((value >> 1) | (u8::from(carry) << 7), carry_out)),
        Mnemonic::Swap => Ok((value.rotate_left(4), carry)),
        _ => Err(unsupported(mnemonic)),
    }
}

pub(super) fn multiply_value(mnemonic: Mnemonic, first: u8, second: u8) -> Result<u16, SimError> {
    let signed = i16::from(first as i8) * i16::from(second as i8);
    let mixed = i16::from(first as i8) * i16::from(second);
    match mnemonic {
        Mnemonic::Mul => Ok(u16::from(first) * u16::from(second)),
        Mnemonic::Muls => Ok(signed as u16),
        Mnemonic::Mulsu => Ok(mixed as u16),
        Mnemonic::Fmul => Ok((u16::from(first) * u16::from(second)) << 1),
        Mnemonic::Fmuls => Ok((signed as u16) << 1),
        Mnemonic::Fmulsu => Ok((mixed as u16) << 1),
        _ => Err(unsupported(mnemonic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_call_and_jump_cycles() {
        assert_eq!(base_cycles(Mnemonic::Eicall), 4);
        assert_eq!(base_cycles(Mnemonic::Eijmp), 2);
    }
}
