//! CMOS standard cells.

#![forbid(unsafe_code)]

mod arithmetic;
mod basic;
mod builder;
mod catalog_bits;
mod catalog_can;
pub mod catalog_digital;
mod catalog_eeprom;
mod catalog_ethernet;
mod catalog_flash;
mod catalog_i2c;
mod catalog_memory;
mod catalog_sd;
mod catalog_spi;
mod catalog_uart;
mod catalog_usb;
mod cell;
mod characterize;
mod digital;
mod registry;
mod sequential;

#[cfg(test)]
mod test_support;

pub use arithmetic::{
    AluFlags, AluResult, add_with_carry, adder8, alu_slice, full_adder, half_adder,
    ripple_carry_adder, sub_with_borrow,
};
pub use basic::{
    and2, buffer, inverter, mux2, nand2, nor2, or2, transmission_gate, tri_state_inverter, xor2,
};
pub use builder::CellBuilder;
pub use cell::Cell;
pub use characterize::{VtcMetrics, noise_margins};
pub use digital::{
    BusDriver, BusPull, BusResolver, CircuitStats, ClockDomain, Comparison, Cpu, CpuTrace,
    DigitalAlu, DigitalBuilder, DigitalCircuit, DigitalModule, DigitalVector, Division, Fifo,
    NetId, OP_ADD, OP_AND, OP_HLT, OP_JMP, OP_JNZ, OP_JZ, OP_LDI, OP_MOV, OP_NOP, OP_OR, OP_SUB,
    OP_XOR, Prescaler, PriorityEncoding, ProgramCounter, RegisterControl, RegisterFile,
    RegisterPorts, ResetDomain, StackPointer, and_all, arithmetic_shift_right, barrel_shift_left,
    barrel_shift_right, bit_reverse, bus_mux, byte_swap, comparator_unsigned, counter,
    counter_loadable, counter_match, crossbar, decoder, decrement, digital_alu, digital_alu_width,
    divider_non_restoring, divider_restoring, encode_alu, encode_branch, encode_hlt, encode_ldi,
    fifo_sync, full_adder as digital_full_adder, funnel_shift_left, funnel_shift_right, increment,
    leading_zero_count, multiplier_array, multiplier_booth, mux_tree, one_hot_encoder, or_all,
    popcount, prescaler, priority_encoder, read_bits, register, register_extended, register_file,
    register_file_masked, reset_synchronizer, resolve_bus, resolve_open_drain, rotate_left,
    rotate_right, saturating_add_unsigned, saturating_sub_unsigned, shift_register,
};
pub use registry::CellKind;
pub use sequential::{d_flip_flop, d_latch, ring_oscillator, sram6t};
