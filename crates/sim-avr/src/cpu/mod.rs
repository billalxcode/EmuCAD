//! AVR execution core with a structural datapath.

use super::adc::Adc;
use super::alu::{ArithmeticUnit, LogicUnit, arithmetic_unit, logic_unit, select_operand};
use super::fetch::ProgramMemory;
use super::gpio::GpioPort;
use super::instruction::{Instruction, Mnemonic, Operand};
use super::multiply::{MultiplyUnit, multiply_unit};
use super::pin::{ExternalPin, PinElectrical, PinElectricalConfig, measure_pin};
use super::register_bank::RegisterBank;
use super::shift::{ShiftUnit, shift_unit};
use super::spi::Spi;
use super::timer0::Timer0;
use super::timer1::Timer1;
use super::timer2::Timer2;
use super::twi::Twi;
use super::uart::Uart;
use sim_cells::{DigitalBuilder, DigitalCircuit, NetId, RegisterPorts, read_bits};
use sim_core::SimError;

/// AVR status register state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sreg {
    /// Carry.
    pub carry: bool,
    /// Zero.
    pub zero: bool,
    /// Negative.
    pub negative: bool,
    /// Overflow.
    pub overflow: bool,
    /// Signed sign.
    pub sign: bool,
    /// Half carry.
    pub half_carry: bool,
    /// Transfer bit.
    pub transfer: bool,
    /// Interrupt enable.
    pub interrupt: bool,
}

impl Sreg {
    /// Returns a flag by AVR SREG index.
    #[must_use]
    pub fn get(self, flag: u8) -> bool {
        match flag {
            0 => self.carry,
            1 => self.zero,
            2 => self.negative,
            3 => self.overflow,
            4 => self.sign,
            5 => self.half_carry,
            6 => self.transfer,
            _ => self.interrupt,
        }
    }

    /// Sets a flag by AVR SREG index.
    pub fn set(&mut self, flag: u8, value: bool) {
        match flag {
            0 => self.carry = value,
            1 => self.zero = value,
            2 => self.negative = value,
            3 => self.overflow = value,
            4 => self.sign = value,
            5 => self.half_carry = value,
            6 => self.transfer = value,
            _ => self.interrupt = value,
        }
    }

    /// Packs the register into an AVR SREG byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        (self.carry as u8)
            | ((self.zero as u8) << 1)
            | ((self.negative as u8) << 2)
            | ((self.overflow as u8) << 3)
            | ((self.sign as u8) << 4)
            | ((self.half_carry as u8) << 5)
            | ((self.transfer as u8) << 6)
            | ((self.interrupt as u8) << 7)
    }

    /// Unpacks an AVR SREG byte.
    #[must_use]
    pub const fn from_byte(value: u8) -> Self {
        Self {
            carry: value & 0x01 != 0,
            zero: value & 0x02 != 0,
            negative: value & 0x04 != 0,
            overflow: value & 0x08 != 0,
            sign: value & 0x10 != 0,
            half_carry: value & 0x20 != 0,
            transfer: value & 0x40 != 0,
            interrupt: value & 0x80 != 0,
        }
    }
}

/// ATmega328P interrupt vector word addresses.
pub const VECTORS: [u16; 26] = [
    0x0000, 0x0002, 0x0004, 0x0006, 0x0008, 0x000A, 0x000C, 0x000E, 0x0010, 0x0012, 0x0014, 0x0016,
    0x0018, 0x001A, 0x001C, 0x001E, 0x0020, 0x0022, 0x0024, 0x0026, 0x0028, 0x002A, 0x002C, 0x002E,
    0x0030, 0x0032,
];

fn vector_address(index: u8) -> u16 {
    VECTORS.get(usize::from(index)).copied().unwrap_or(0)
}

/// One executed instruction trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvrTrace {
    /// Program counter after the instruction.
    pub pc: u16,
    /// Executed instruction.
    pub instruction: Instruction,
    /// Register snapshot.
    pub registers: [u8; 32],
    /// Status register snapshot.
    pub sreg: Sreg,
    /// Halt state.
    pub halted: bool,
    /// Cycles used by this instruction.
    pub cycles: u8,
    /// Total cycles since reset.
    pub total_cycles: u64,
}

struct AluNets {
    first: Vec<NetId>,
    second: Vec<NetId>,
    immediate: Vec<NetId>,
    use_immediate: NetId,
    subtract: NetId,
    carry: NetId,
}

#[derive(Clone, Copy)]
struct MemoryMode {
    base: u8,
    post: bool,
    pre: bool,
    load: bool,
}

struct Gpio {
    b: GpioPort,
    c: GpioPort,
    d: GpioPort,
}

struct Timers {
    timer0: Timer0,
    timer1: Timer1,
    timer2: Timer2,
}

struct Serial {
    spi: Spi,
    twi: Twi,
    adc: Adc,
}

impl Serial {
    fn new() -> Self {
        Self {
            spi: Spi::new(),
            twi: Twi::new(),
            adc: Adc::new(),
        }
    }
}

struct Datapath {
    bank: RegisterBank,
    arithmetic: ArithmeticUnit,
    logic: LogicUnit,
    shift: ShiftUnit,
    multiply: MultiplyUnit,
    gpio: Gpio,
    timers: Timers,
    serial: Serial,
    uart: Uart,
}

/// A small AVR core for instruction integration tests.
pub struct Cpu {
    circuit: DigitalCircuit,
    datapath: Datapath,
    program: ProgramMemory,
    pc: u16,
    sreg: Sreg,
    halted: bool,
    io: [u8; 256],
    data: Vec<u8>,
    sp: u16,
    pending: u32,
    sei_delay: u8,
    sleeping: bool,
    cycles: u64,
    pin_change: PinChange,
}

#[derive(Default)]
struct PinChange {
    pcicr: u8,
    pcifr: u8,
    pcmsk: [u8; 3],
    last_pin: [u8; 3],
}

struct Units {
    arithmetic: ArithmeticUnit,
    logic: LogicUnit,
    shift: ShiftUnit,
    multiply: MultiplyUnit,
}

mod arithmetic;
mod build;
mod execute;
mod helpers;
mod io;
mod state;
mod step;
#[cfg(test)]
mod tests;

use build::*;
use helpers::*;
