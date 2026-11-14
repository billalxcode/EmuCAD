//! STK500/Optiboot bootloader protocol for the virtual serial port.

const FLASH_SIZE: usize = 32 * 1024;
const SIGNATURE: [u8; 3] = [0x1E, 0x95, 0x0F];

const CMD_GET_SYNC: u8 = 0x30;
const CMD_GET_PARAMETER: u8 = 0x41;
const CMD_SET_DEVICE: u8 = 0x42;
const CMD_SET_DEVICE_EXT: u8 = 0x45;
const CMD_ENTER_PROGMODE: u8 = 0x50;
const CMD_LEAVE_PROGMODE: u8 = 0x51;
const CMD_CHIP_ERASE: u8 = 0x52;
const CMD_CHECK_AUTOINC: u8 = 0x53;
const CMD_LOAD_ADDRESS: u8 = 0x55;
const CMD_UNIVERSAL: u8 = 0x56;
const CMD_PROG_PAGE: u8 = 0x64;
const CMD_READ_OSCCAL: u8 = 0x66;
const CMD_READ_PAGE: u8 = 0x74;
const CMD_READ_SIGN: u8 = 0x75;

const RESP_INSYNC: u8 = 0x14;
const RESP_OK: u8 = 0x10;
const RESP_NOSYNC: u8 = 0x00;
const STK500_EOP: u8 = 0x20;

/// One STK500 frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Command byte.
    pub command: u8,
    /// Sequence byte.
    pub sequence: u8,
    /// Command payload.
    pub data: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Header {
    command: u8,
    sequence: u8,
    complement: u8,
}

/// Frame parsing error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// Checksum, sequence complement, or STK500 end-of-packet is invalid.
    InvalidChecksum,
}

/// Parses one frame from a byte buffer.
pub fn parse_frame(buffer: &mut Vec<u8>) -> Result<Option<Frame>, FrameError> {
    if buffer.is_empty() {
        return Ok(None);
    }
    if buffer[0] == 0x1B {
        return parse_avr109(buffer);
    }
    parse_stk500(buffer)
}

mod frame;
mod protocol;
#[cfg(test)]
mod tests;

use frame::{parse_avr109, parse_stk500};
pub use protocol::Bootloader;
