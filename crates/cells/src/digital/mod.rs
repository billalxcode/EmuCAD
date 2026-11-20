//! Two-state digital logic and digital blocks.

pub mod alu;
pub mod blocks;
pub mod bus;
pub mod circuit;
pub mod cpu;
pub mod module;
pub mod registers;
pub mod reset;
pub mod shifter;
pub mod vector;

pub use alu::{DigitalAlu, digital_alu, digital_alu_width};
pub use blocks::{
    ProgramCounter, StackPointer, and_all, counter, decoder, decrement, full_adder, increment,
    mux_tree, or_all, read_bits, register, shift_register,
};
pub use bus::{BusDriver, BusPull, BusResolver, resolve_bus, resolve_open_drain};
pub use circuit::{CircuitStats, ClockDomain, DigitalBuilder, DigitalCircuit, NetId, ResetDomain};
pub use cpu::{
    Cpu, CpuTrace, OP_ADD, OP_AND, OP_HLT, OP_JMP, OP_JNZ, OP_JZ, OP_LDI, OP_MOV, OP_NOP, OP_OR,
    OP_SUB, OP_XOR, encode_alu, encode_branch, encode_hlt, encode_ldi,
};
pub use module::DigitalModule;
pub use registers::{RegisterFile, RegisterPorts, register_file, register_file_masked};
pub use reset::reset_synchronizer;
pub use shifter::{
    arithmetic_shift_right, barrel_shift_left, barrel_shift_right, funnel_shift_left,
    funnel_shift_right, rotate_left, rotate_right,
};
pub use vector::DigitalVector;
