//! Digital building blocks for registers and counters.

use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use kernel::SimError;

mod combinational;
mod pointers;
mod registers;
#[cfg(test)]
mod tests;

pub use combinational::{
    add_vectors, and_all, decoder, mux_tree, or_all, read_bits, validate_operands,
};
pub use pointers::{ProgramCounter, StackPointer};
pub use registers::{
    counter, decrement, full_adder, increment, register, register_bit, shift_register,
};
