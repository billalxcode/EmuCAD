use super::*;

pub struct Bootloader {
    flash: Vec<u8>,
    address: usize,
    running: bool,
}

impl Bootloader {
    /// Creates a bootloader with erased flash.
    #[must_use]
    pub fn new() -> Self {
        Self {
            flash: vec![0xFF; FLASH_SIZE],
            address: 0,
            running: false,
        }
    }

    /// Returns true after the uploader leaves programming mode.
    #[must_use]
    pub fn running(&self) -> bool {
        self.running
    }

    /// Handles one frame and returns the response bytes.
    pub fn handle(&mut self, frame: &Frame) -> Vec<u8> {
        if let Some(response) = fixed_response(frame) {
            return response;
        }
        match frame.command {
            CMD_GET_SYNC => self.enter_programming(),
            CMD_LEAVE_PROGMODE => self.leave_programming(),
            CMD_CHIP_ERASE => self.erase(),
            CMD_LOAD_ADDRESS => self.load_address(&frame.data),
            CMD_PROG_PAGE => self.program_page(&frame.data),
            CMD_READ_PAGE => self.read_page(&frame.data),
            _ => error(),
        }
    }

    fn enter_programming(&mut self) -> Vec<u8> {
        self.running = false;
        ok(&[])
    }

    fn leave_programming(&mut self) -> Vec<u8> {
        self.running = true;
        ok(&[])
    }

    /// Returns the uploaded flash image as instruction words.
    #[must_use]
    pub fn program_words(&self) -> Vec<u16> {
        let end = self.used_end();
        let mut words: Vec<u16> = self.flash[..end]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        if words.is_empty() {
            words.push(0xCFFF);
        }
        words
    }

    fn used_end(&self) -> usize {
        let mut end = self.flash.len();
        while end >= 2 && self.flash[end - 2] == 0xFF && self.flash[end - 1] == 0xFF {
            end -= 2;
        }
        end
    }

    fn erase(&mut self) -> Vec<u8> {
        self.flash.fill(0xFF);
        self.address = 0;
        ok(&[])
    }

    fn load_address(&mut self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return error();
        }
        self.address = usize::from(u16::from_le_bytes([data[0], data[1]]));
        ok(&[])
    }

    fn program_page(&mut self, data: &[u8]) -> Vec<u8> {
        let Some((length, memory)) = page_header(data) else {
            return error();
        };
        if data.len() < 3 + length {
            return error();
        }
        if memory == b'F' {
            self.write_flash(&data[3..3 + length]);
        }
        ok(&[])
    }

    fn write_flash(&mut self, data: &[u8]) {
        let start = self.address.saturating_mul(2);
        let end = start.saturating_add(data.len()).min(self.flash.len());
        if start >= end {
            return;
        }
        let available = &data[..end - start];
        self.flash[start..end].copy_from_slice(available);
        self.address = self.address.saturating_add(data.len() / 2);
    }

    fn read_page(&self, data: &[u8]) -> Vec<u8> {
        let Some((length, memory)) = page_header(data) else {
            return error();
        };
        if memory != b'F' {
            return ok(&[]);
        }
        let start = self.address.saturating_mul(2);
        let end = start.saturating_add(length).min(self.flash.len());
        if start >= end {
            return error();
        }
        ok(&self.flash[start..end])
    }
}

fn page_header(data: &[u8]) -> Option<(usize, u8)> {
    if data.len() < 3 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([data[0], data[1]]));
    Some((length, data[2]))
}

fn fixed_response(frame: &Frame) -> Option<Vec<u8>> {
    match frame.command {
        CMD_GET_PARAMETER => Some(ok(&[parameter(frame.data.first().copied().unwrap_or(0))])),
        CMD_SET_DEVICE | CMD_SET_DEVICE_EXT | CMD_ENTER_PROGMODE | CMD_CHECK_AUTOINC => {
            Some(ok(&[]))
        }
        CMD_UNIVERSAL => Some(ok(&[0x00])),
        CMD_READ_SIGN => Some(ok(&SIGNATURE)),
        CMD_READ_OSCCAL => Some(ok(&[0x00])),
        _ => None,
    }
}

fn parameter(code: u8) -> u8 {
    match code {
        0x80 => 0x03,
        0x81 => 0x04,
        0x82 => 0x04,
        0x93 => 0x03,
        _ => 0x00,
    }
}

fn ok(data: &[u8]) -> Vec<u8> {
    let mut response = Vec::with_capacity(data.len() + 2);
    response.push(RESP_INSYNC);
    response.extend_from_slice(data);
    response.push(RESP_OK);
    response
}

fn error() -> Vec<u8> {
    vec![RESP_INSYNC, RESP_NOSYNC]
}
