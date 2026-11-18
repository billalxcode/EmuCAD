//! AVR opcode decoder.

use super::instruction::{Instruction, Mnemonic, Operand};
use kernel::SimError;

/// Returns true when the instruction occupies two words.
#[must_use]
pub fn uses_second_word(first: u16) -> bool {
    matches!(first & 0xFE0F, 0x9000 | 0x9200) || matches!(first & 0xFE0E, 0x940C | 0x940E)
}

/// Decodes one AVR instruction.
pub fn decode(first: u16, second: Option<u16>) -> Result<Instruction, SimError> {
    match first & 0xF000 {
        0x0000 => decode_group_0(first),
        0x1000 => decode_group_1(first),
        0x2000 => decode_group_2(first),
        0x3000 => Ok(immediate(first, Mnemonic::Cpi)),
        0x4000 => Ok(immediate(first, Mnemonic::Sbci)),
        0x5000 => Ok(immediate(first, Mnemonic::Subi)),
        0x6000 => Ok(immediate(first, Mnemonic::Ori)),
        0x7000 => Ok(immediate(first, Mnemonic::Andi)),
        0x8000 | 0xA000 => decode_ldd(first),
        0x9000 => decode_group_9(first, second),
        0xB000 => decode_in_out(first),
        0xC000 => Ok(relative(first, Mnemonic::Rjmp)),
        0xD000 => Ok(relative(first, Mnemonic::Rcall)),
        0xE000 => Ok(immediate(first, Mnemonic::Ldi)),
        0xF000 => decode_group_f(first),
        _ => Err(illegal(first)),
    }
}
fn register_index(first: u16) -> u8 {
    ((first >> 4) & 0x1F) as u8
}

fn sign_extend(value: u16, bits: u16) -> i16 {
    let shift = 16 - bits;
    ((value << shift) as i16) >> shift
}

fn illegal(first: u16) -> SimError {
    SimError::Parse(format!("illegal AVR opcode 0x{first:04X}"))
}

fn truncated(first: u16) -> SimError {
    SimError::Parse(format!("missing second word for 0x{first:04X}"))
}

mod builders;
mod groups;
#[cfg(test)]
mod tests;

use builders::*;
use groups::*;
