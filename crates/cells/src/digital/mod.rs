//! Two-state digital logic and digital blocks.

pub mod alu;
pub mod blocks;
pub mod bus;
pub mod circuit;
pub mod cpu;
pub mod module;
pub mod registers;
pub mod vector;

pub use alu::{DigitalAlu, digital_alu};
pub use blocks::{
    ProgramCounter, StackPointer, and_all, counter, decoder, decrement, full_adder, increment,
    mux_tree, or_all, read_bits, register, shift_register,
};
pub use bus::{BusDriver, BusPull, BusResolver, resolve_bus, resolve_open_drain};
pub use circuit::{CircuitStats, DigitalBuilder, DigitalCircuit, NetId};
pub use cpu::{
    Cpu, CpuTrace, OP_ADD, OP_AND, OP_HLT, OP_JMP, OP_JNZ, OP_JZ, OP_LDI, OP_MOV, OP_NOP, OP_OR,
    OP_SUB, OP_XOR, encode_alu, encode_branch, encode_hlt, encode_ldi,
};
pub use module::DigitalModule;
pub use registers::{RegisterFile, RegisterPorts, register_file};
pub use vector::DigitalVector;
