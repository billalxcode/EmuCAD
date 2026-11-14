//! Small multi-cycle CPU for digital integration tests.

use super::alu::{DigitalAlu, digital_alu};
use super::blocks::{ProgramCounter, decoder, or_all, read_bits, register};
use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use super::registers::{RegisterFile, RegisterPorts, register_file};
use sim_core::SimError;

/// No operation.
pub const OP_NOP: u16 = 0;
/// Load immediate 6-bit value.
pub const OP_LDI: u16 = 1;
/// Add register.
pub const OP_ADD: u16 = 2;
/// Subtract register.
pub const OP_SUB: u16 = 3;
/// Bitwise AND.
pub const OP_AND: u16 = 4;
/// Bitwise OR.
pub const OP_OR: u16 = 5;
/// Bitwise XOR.
pub const OP_XOR: u16 = 6;
/// Move register.
pub const OP_MOV: u16 = 7;
/// Unconditional jump.
pub const OP_JMP: u16 = 8;
/// Jump if zero.
pub const OP_JZ: u16 = 9;
/// Jump if not zero.
pub const OP_JNZ: u16 = 10;
/// Halt.
pub const OP_HLT: u16 = 31;

const WRITE_OPCODES: [usize; 7] = [1, 2, 3, 4, 5, 6, 7];
const FLAG_OPCODES: [usize; 5] = [2, 3, 4, 5, 6];
const ALU_BIT0: [usize; 4] = [1, 3, 5, 7];
const ALU_BIT1: [usize; 3] = [4, 5, 6];
const ALU_BIT2: [usize; 3] = [1, 6, 7];

/// Encodes a load-immediate instruction.
#[must_use]
pub fn encode_ldi(register: u16, value: u16) -> u16 {
    (OP_LDI << 11) | ((register & 0x1F) << 6) | (value & 0x3F)
}

/// Encodes an ALU register instruction.
#[must_use]
pub fn encode_alu(opcode: u16, destination: u16, source: u16) -> u16 {
    (opcode << 11) | ((destination & 0x1F) << 6) | (source & 0x1F)
}

/// Encodes a jump instruction.
#[must_use]
pub fn encode_branch(opcode: u16, target: u16) -> u16 {
    (opcode << 11) | (target & 0x7FF)
}

/// Encodes a halt instruction.
#[must_use]
pub const fn encode_hlt() -> u16 {
    OP_HLT << 11
}

/// One instruction trace entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuTrace {
    /// Program counter after the instruction.
    pub pc: u64,
    /// Instruction word in the register.
    pub instruction: u16,
    /// Halt state.
    pub halted: bool,
    /// Register snapshot after the instruction.
    pub registers: [u8; 32],
}

struct Fields {
    opcode: Vec<NetId>,
    rd: Vec<NetId>,
    rs: Vec<NetId>,
    imm: Vec<NetId>,
    target: Vec<NetId>,
}

struct Decode {
    fields: Fields,
    lines: Vec<NetId>,
    fetch: NetId,
    execute: NetId,
    signals: DecodeSignals,
}

struct DecodeSignals {
    reg_write: NetId,
    flag_write: NetId,
    use_imm: NetId,
    alu_op: Vec<NetId>,
}

struct Branch {
    pc_load: NetId,
    halt: NetId,
}

struct BuiltCpu {
    registers: RegisterFile,
    pc: ProgramCounter,
    ir: Vec<NetId>,
    halted: NetId,
}

/// A small CPU with 32 registers, 8-bit ALU, and 16-bit PC.
pub struct Cpu {
    circuit: DigitalCircuit,
    registers: RegisterFile,
    pc: ProgramCounter,
    ir: Vec<NetId>,
    halted: NetId,
    instruction: Vec<NetId>,
    program: Vec<u16>,
}

impl Cpu {
    /// Builds a CPU with the provided program.
    pub fn new(program: Vec<u16>) -> Result<Self, SimError> {
        if program.is_empty() || program.len() > 2048 {
            return Err(SimError::InvalidNetlist(
                "program must contain 1..=2048 words".to_string(),
            ));
        }
        let mut builder = DigitalBuilder::new();
        let instruction = instruction_inputs(&mut builder);
        let built = build_circuit(&mut builder, &instruction)?;
        Ok(Self {
            circuit: builder.build(),
            registers: built.registers,
            pc: built.pc,
            ir: built.ir,
            halted: built.halted,
            instruction,
            program,
        })
    }

    /// Executes one instruction and returns its trace.
    pub fn step(&mut self) -> Result<Option<CpuTrace>, SimError> {
        if self.is_halted() {
            return Ok(None);
        }
        self.fetch()?;
        self.circuit.tick()?;
        self.circuit.tick()?;
        Ok(Some(self.trace()?))
    }

    /// Executes until halt or until the step limit is reached.
    pub fn run(&mut self, max_steps: usize) -> Result<Vec<CpuTrace>, SimError> {
        let mut traces = Vec::new();
        for _ in 0..max_steps {
            match self.step()? {
                Some(trace) => traces.push(trace),
                None => return Ok(traces),
            }
        }
        Err(SimError::ConvergenceFailure {
            iteration: max_steps,
        })
    }

    /// Returns a register value.
    #[must_use]
    pub fn register(&self, index: usize) -> Option<u8> {
        let bits = self.registers.bits().get(index)?;
        Some(read_bits(&self.circuit, bits) as u8)
    }

    /// Returns the program counter.
    #[must_use]
    pub fn pc(&self) -> u64 {
        self.pc.read(&self.circuit)
    }

    /// Returns the instruction register value.
    #[must_use]
    pub fn instruction(&self) -> u16 {
        read_bits(&self.circuit, &self.ir) as u16
    }

    /// Returns true when halted.
    #[must_use]
    pub fn is_halted(&self) -> bool {
        self.circuit.value(self.halted)
    }

    /// Resets the CPU to its initial state.
    pub fn reset(&mut self) -> Result<(), SimError> {
        self.circuit.reset()?;
        self.set_instruction(0)?;
        self.circuit.settle()
    }

    fn fetch(&mut self) -> Result<(), SimError> {
        let address = (self.pc.read(&self.circuit) as usize) & 0x7FF;
        let word = self.program.get(address).copied().unwrap_or(0);
        self.set_instruction(word)?;
        self.circuit.settle()
    }

    fn set_instruction(&mut self, word: u16) -> Result<(), SimError> {
        for index in 0..self.instruction.len() {
            let bit = word & (1 << index) != 0;
            self.circuit.set_input(&format!("instr{index}"), bit)?;
        }
        Ok(())
    }

    fn trace(&self) -> Result<CpuTrace, SimError> {
        let mut registers = [0u8; 32];
        for (index, bits) in self.registers.bits().iter().enumerate() {
            registers[index] = read_bits(&self.circuit, bits) as u8;
        }
        Ok(CpuTrace {
            pc: self.pc.read(&self.circuit),
            instruction: read_bits(&self.circuit, &self.ir) as u16,
            halted: self.is_halted(),
            registers,
        })
    }
}

mod build;
#[cfg(test)]
mod tests;

use build::*;
