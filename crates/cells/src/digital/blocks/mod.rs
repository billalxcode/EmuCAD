//! Digital building blocks for registers and counters.

use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use kernel::SimError;

mod combinational;
mod pointers;
mod registers;
#[cfg(test)]
mod tests;

pub use combinational::{
    add_vectors, and_all, decoder, invert_vector, mux_tree, or_all, read_bits, select_vector,
    validate_operands, validate_pair, validate_power_of_two, validate_width,
};
pub use pointers::{ProgramCounter, StackPointer};
pub use registers::{
    RegisterControl, counter, decrement, full_adder, increment, register, register_bit,
    register_extended, shift_register,
};
