use super::{decode, uses_second_word};
use crate::instruction::{Instruction, Mnemonic, Operand};

fn two_reg(base: u16, d: u8, r: u8) -> u16 {
    base | ((u16::from(r & 0x10)) << 5) | (u16::from(d) << 4) | u16::from(r & 0xF)
}

fn immediate(base: u16, d: u8, k: u8) -> u16 {
    base | (u16::from(k & 0xF0) << 4) | (u16::from(d - 16) << 4) | u16::from(k & 0xF)
}

fn branch(base: u16, flag: u8, offset: i8) -> u16 {
    base | ((u16::from(offset as u8) & 0x7F) << 3) | u16::from(flag)
}

fn relative(base: u16, offset: i16) -> u16 {
    base | (offset as u16 & 0x0FFF)
}

fn reg(mnemonic: Mnemonic, d: u8, r: u8) -> Instruction {
    Instruction::new(mnemonic, Operand::Register(d), Operand::Register(r), 1)
}

fn one(mnemonic: Mnemonic, d: u8) -> Instruction {
    Instruction::new(mnemonic, Operand::Register(d), Operand::None, 1)
}

#[test]
fn decodes_arithmetic_register_ops() {
    let cases = [
        (0x0C00, Mnemonic::Add),
        (0x1C00, Mnemonic::Adc),
        (0x1800, Mnemonic::Sub),
        (0x0800, Mnemonic::Sbc),
        (0x2000, Mnemonic::And),
        (0x2800, Mnemonic::Or),
        (0x2400, Mnemonic::Eor),
        (0x1400, Mnemonic::Cp),
        (0x0400, Mnemonic::Cpc),
        (0x1000, Mnemonic::Cpse),
        (0x2C00, Mnemonic::Mov),
        (0x9C00, Mnemonic::Mul),
    ];
    for (base, mnemonic) in cases {
        let word = two_reg(base, 17, 22);
        assert_eq!(decode(word, None).expect("decode"), reg(mnemonic, 17, 22));
    }
}

#[test]
fn decodes_immediate_ops() {
    let cases = [
        (0xE000, Mnemonic::Ldi),
        (0x3000, Mnemonic::Cpi),
        (0x5000, Mnemonic::Subi),
        (0x4000, Mnemonic::Sbci),
        (0x7000, Mnemonic::Andi),
        (0x6000, Mnemonic::Ori),
    ];
    for (base, mnemonic) in cases {
        let word = immediate(base, 21, 0xA5);
        let expected =
            Instruction::new(mnemonic, Operand::Register(21), Operand::Immediate(0xA5), 1);
        assert_eq!(decode(word, None).expect("decode"), expected);
    }
}

#[test]
fn decodes_single_register_ops() {
    let cases = [
        (0x9400, Mnemonic::Com),
        (0x9401, Mnemonic::Neg),
        (0x9402, Mnemonic::Swap),
        (0x9403, Mnemonic::Inc),
        (0x9405, Mnemonic::Asr),
        (0x9406, Mnemonic::Lsr),
        (0x9407, Mnemonic::Ror),
        (0x940A, Mnemonic::Dec),
    ];
    for (base, mnemonic) in cases {
        let word = base | (u16::from(13u8) << 4);
        assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 13));
    }
}

#[test]
fn decodes_branches_and_relative_jumps() {
    let word = branch(0xF000, 1, -5);
    let expected = Instruction::new(Mnemonic::Brbs, Operand::Flag(1), Operand::Offset(-5), 1);
    assert_eq!(decode(word, None).expect("brbs"), expected);
    let word = branch(0xF400, 2, 9);
    let expected = Instruction::new(Mnemonic::Brbc, Operand::Flag(2), Operand::Offset(9), 1);
    assert_eq!(decode(word, None).expect("brbc"), expected);
    let word = relative(0xC000, -2);
    assert_eq!(
        decode(word, None).expect("rjmp"),
        Instruction::new(Mnemonic::Rjmp, Operand::Offset(-2), Operand::None, 1)
    );
    let word = relative(0xD000, 7);
    assert_eq!(
        decode(word, None).expect("rcall"),
        Instruction::new(Mnemonic::Rcall, Operand::Offset(7), Operand::None, 1)
    );
}

#[test]
fn decodes_ldd_and_std() {
    let word = 0x8000 | (u16::from(7u8) << 4) | 0x0003;
    let expected = Instruction::new(
        Mnemonic::LddZ,
        Operand::Register(7),
        Operand::Displacement(3),
        1,
    );
    assert_eq!(decode(word, None).expect("ldd"), expected);
    let word = 0x8208 | (u16::from(4u8) << 4) | 0x0005;
    let expected = Instruction::new(
        Mnemonic::StdY,
        Operand::Displacement(5),
        Operand::Register(4),
        1,
    );
    assert_eq!(decode(word, None).expect("std"), expected);
}

#[test]
fn decodes_lds_and_sts() {
    let word = 0x9000 | (u16::from(9u8) << 4);
    let expected = Instruction::new(
        Mnemonic::Lds,
        Operand::Register(9),
        Operand::Address(0x1234),
        2,
    );
    assert_eq!(decode(word, Some(0x1234)).expect("lds"), expected);
    let word = 0x9200 | (u16::from(9u8) << 4);
    let expected = Instruction::new(
        Mnemonic::Sts,
        Operand::Register(9),
        Operand::Address(0x4321),
        2,
    );
    assert_eq!(decode(word, Some(0x4321)).expect("sts"), expected);
}

#[test]
fn decodes_indirect_loads() {
    let cases = [
        (0x900C, Mnemonic::LdX),
        (0x900D, Mnemonic::LdXPost),
        (0x900E, Mnemonic::LdXPre),
        (0x9009, Mnemonic::LdYPost),
        (0x900A, Mnemonic::LdYPre),
        (0x9001, Mnemonic::LdZPost),
        (0x9002, Mnemonic::LdZPre),
        (0x900F, Mnemonic::Pop),
    ];
    for (base, mnemonic) in cases {
        let word = base | (u16::from(11u8) << 4);
        assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 11));
    }
}

#[test]
fn decodes_indirect_stores() {
    let cases = [
        (0x920C, Mnemonic::StX),
        (0x920D, Mnemonic::StXPost),
        (0x920E, Mnemonic::StXPre),
        (0x9209, Mnemonic::StYPost),
        (0x920A, Mnemonic::StYPre),
        (0x9201, Mnemonic::StZPost),
        (0x9202, Mnemonic::StZPre),
        (0x920F, Mnemonic::Push),
    ];
    for (base, mnemonic) in cases {
        let word = base | (u16::from(11u8) << 4);
        assert_eq!(decode(word, None).expect("decode"), one(mnemonic, 11));
    }
}

#[test]
fn decodes_io_and_bit_ops() {
    let word = 0xB400 | (u16::from(5u8) << 4) | 0x0003;
    let expected = Instruction::new(Mnemonic::In, Operand::Register(5), Operand::Io(0x23), 1);
    assert_eq!(decode(word, None).expect("in"), expected);
    let word = 0xBC00 | (u16::from(5u8) << 4) | 0x0003;
    let expected = Instruction::new(Mnemonic::Out, Operand::Io(0x23), Operand::Register(5), 1);
    assert_eq!(decode(word, None).expect("out"), expected);
    let word = 0x9A00 | (u16::from(12u8) << 3) | 0x0006;
    let expected = Instruction::new(Mnemonic::Sbi, Operand::Io(12), Operand::Bit(6), 1);
    assert_eq!(decode(word, None).expect("sbi"), expected);
}

#[test]
fn decodes_long_jump_and_call() {
    let expected = Instruction::new(
        Mnemonic::Jmp,
        Operand::LongAddress(0x001234),
        Operand::None,
        2,
    );
    assert_eq!(decode(0x940C, Some(0x1234)).expect("jmp"), expected);
    let expected = Instruction::new(
        Mnemonic::Call,
        Operand::LongAddress(0x011234),
        Operand::None,
        2,
    );
    assert_eq!(decode(0x940F, Some(0x1234)).expect("call"), expected);
}

#[test]
fn decodes_fixed_ops() {
    let cases = [
        (0x0000, Mnemonic::Nop),
        (0x9588, Mnemonic::Sleep),
        (0x9598, Mnemonic::Break),
        (0x95A8, Mnemonic::Wdr),
        (0x95E8, Mnemonic::Spm),
        (0x9508, Mnemonic::Ret),
        (0x9518, Mnemonic::Reti),
        (0x9409, Mnemonic::Ijmp),
        (0x9419, Mnemonic::Eijmp),
        (0x9509, Mnemonic::Icall),
        (0x9519, Mnemonic::Eicall),
    ];
    for (word, mnemonic) in cases {
        let expected = Instruction::new(mnemonic, Operand::None, Operand::None, 1);
        assert_eq!(decode(word, None).expect("decode"), expected);
    }
}

#[test]
fn decodes_flag_and_bit_ops() {
    let expected = Instruction::new(Mnemonic::Bset, Operand::Flag(3), Operand::None, 1);
    assert_eq!(decode(0x9438, None).expect("bset"), expected);
    let expected = Instruction::new(Mnemonic::Bclr, Operand::Flag(3), Operand::None, 1);
    assert_eq!(decode(0x94B8, None).expect("bclr"), expected);
    let word = 0xF800 | (u16::from(6u8) << 4) | 0x0002;
    let expected = Instruction::new(Mnemonic::Bld, Operand::Register(6), Operand::Bit(2), 1);
    assert_eq!(decode(word, None).expect("bld"), expected);
    let word = 0xFC00 | (u16::from(6u8) << 4) | 0x0002;
    let expected = Instruction::new(Mnemonic::Sbrc, Operand::Register(6), Operand::Bit(2), 1);
    assert_eq!(decode(word, None).expect("sbrc"), expected);
}

#[test]
fn detects_second_word_and_rejects_truncated() {
    assert!(uses_second_word(0x9000));
    assert!(uses_second_word(0x940C));
    assert!(!uses_second_word(0x0C00));
    assert!(decode(0x9000, None).is_err());
    assert!(decode(0x940C, None).is_err());
}

#[test]
fn every_word_decodes_without_panic() {
    for word in 0..=u16::MAX {
        let _ = decode(word, Some(0));
    }
}

#[test]
fn decoded_instructions_have_valid_ranges() {
    for word in 0..=u16::MAX {
        let Ok(instruction) = decode(word, Some(0)) else {
            continue;
        };
        assert_eq!(instruction.words == 2, uses_second_word(word));
        for operand in [instruction.first, instruction.second] {
            match operand {
                Operand::Register(value) => assert!(value < 32),
                Operand::Bit(value) | Operand::Flag(value) => assert!(value < 8),
                Operand::Io(value) => assert!(value < 64),
                Operand::Displacement(value) => assert!(value < 64),
                _ => {}
            }
        }
    }
}
