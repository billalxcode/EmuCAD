use super::{Cpu, Sreg, VECTORS};
use crate::pin::{ExternalPin, PinElectricalConfig};

fn ldi(register: u16, value: u16) -> u16 {
    0xE000 | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
}

fn reg_op(base: u16, destination: u16, source: u16) -> u16 {
    base | ((source & 0x10) << 5) | ((destination & 0x1F) << 4) | (source & 0x0F)
}

fn one_reg(base: u16, register: u16) -> u16 {
    base | ((register & 0x1F) << 4)
}

fn branch(flag: u16, offset: i8) -> u16 {
    0xF000 | (((offset as u16) & 0x7F) << 3) | flag
}

fn branch_clear(flag: u16, offset: i8) -> u16 {
    0xF400 | (((offset as u16) & 0x7F) << 3) | flag
}

fn immediate(base: u16, register: u16, value: u16) -> u16 {
    base | ((value & 0xF0) << 4) | ((register - 16) << 4) | (value & 0x0F)
}

fn in_out(base: u16, register: u16, address: u16) -> u16 {
    base | (((address >> 4) & 3) << 9) | ((register & 0x1F) << 4) | (address & 0x0F)
}

fn io_bit_op(base: u16, address: u16, bit: u16) -> u16 {
    base | ((address & 0x1F) << 3) | (bit & 0x07)
}

fn bit_register(base: u16, register: u16, bit: u16) -> u16 {
    base | ((register & 0x1F) << 4) | (bit & 0x07)
}

fn sreg_flag(base: u16, flag: u16) -> u16 {
    base | ((flag & 0x07) << 4)
}

fn run(program: Vec<u16>) -> Cpu {
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.run(100).expect("run");
    cpu
}

mod arithmetic;
mod control;
mod core;
mod gpio;
mod memory;
mod peripherals;
mod pure_paths;
mod serial;
mod timer;
