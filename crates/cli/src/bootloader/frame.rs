use super::*;

pub(super) fn parse_avr109(buffer: &mut Vec<u8>) -> Result<Option<Frame>, FrameError> {
    resync_avr109(buffer);
    if buffer.len() < 4 {
        return Ok(None);
    }
    let header = parse_header(buffer)?;
    let Some(total) = frame_length(header.command, &buffer[4..]) else {
        return Ok(None);
    };
    if buffer.len() < total {
        return Ok(None);
    }
    finish_frame(buffer, header, total)
}

pub(super) fn parse_stk500(buffer: &mut Vec<u8>) -> Result<Option<Frame>, FrameError> {
    let Some((command, payload, total)) = stk500_frame(buffer) else {
        return Ok(None);
    };
    if buffer[total - 1] != STK500_EOP {
        buffer.remove(0);
        return Err(FrameError::InvalidChecksum);
    }
    let data = buffer[1..1 + payload].to_vec();
    buffer.drain(..total);
    Ok(Some(Frame {
        command,
        sequence: 0,
        data,
    }))
}

fn stk500_frame(buffer: &[u8]) -> Option<(u8, usize, usize)> {
    let command = *buffer.first()?;
    let payload = payload_length(command, buffer.get(1..)?)?;
    let total = 1 + payload + 1;
    (buffer.len() >= total).then_some((command, payload, total))
}

fn parse_header(buffer: &mut Vec<u8>) -> Result<Header, FrameError> {
    let header = Header {
        command: buffer[1],
        sequence: buffer[2],
        complement: buffer[3],
    };
    if header.sequence ^ header.complement != 0xFF {
        buffer.remove(0);
        return Err(FrameError::InvalidChecksum);
    }
    Ok(header)
}

fn frame_length(command: u8, available: &[u8]) -> Option<usize> {
    let payload = payload_length(command, available)?;
    Some(4 + payload + 1)
}

fn finish_frame(
    buffer: &mut Vec<u8>,
    header: Header,
    total: usize,
) -> Result<Option<Frame>, FrameError> {
    let data = buffer[4..total - 1].to_vec();
    let checksum = buffer[total - 1];
    buffer.drain(..total);
    if checksum != frame_checksum(header.command, header.sequence, header.complement, &data) {
        return Err(FrameError::InvalidChecksum);
    }
    Ok(Some(Frame {
        command: header.command,
        sequence: header.sequence,
        data,
    }))
}

fn resync_avr109(buffer: &mut Vec<u8>) {
    match buffer.iter().position(|byte| *byte == 0x1B) {
        Some(0) => {}
        Some(index) => {
            buffer.drain(..index);
        }
        None => buffer.clear(),
    }
}

fn payload_length(command: u8, available: &[u8]) -> Option<usize> {
    match command {
        CMD_GET_PARAMETER => Some(1),
        CMD_SET_DEVICE => Some(20),
        CMD_SET_DEVICE_EXT => Some(5),
        CMD_LOAD_ADDRESS => Some(2),
        CMD_UNIVERSAL => Some(4),
        CMD_PROG_PAGE => variable_length(available),
        CMD_READ_PAGE => Some(3),
        _ => Some(0),
    }
}

fn variable_length(available: &[u8]) -> Option<usize> {
    if available.len() < 3 {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([available[0], available[1]]));
    Some(3 + length)
}

fn frame_checksum(command: u8, sequence: u8, complement: u8, data: &[u8]) -> u8 {
    data.iter()
        .fold(command ^ sequence ^ complement, |value, byte| value ^ byte)
}
