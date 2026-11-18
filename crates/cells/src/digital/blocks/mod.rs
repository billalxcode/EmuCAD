//! Digital building blocks for registers and counters.

use super::circuit::{DigitalBuilder, DigitalCircuit, NetId};
use kernel::SimError;

mod combinational;
mod pointers;
mod registers;
#[cfg(test)]
mod tests;

pub use combinational::{and_all, decoder, mux_tree, or_all, read_bits};
pub use pointers::{ProgramCounter, StackPointer};
pub use registers::{counter, decrement, full_adder, increment, register, shift_register};
