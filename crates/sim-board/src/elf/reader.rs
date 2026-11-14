use super::*;

pub(super) fn read_string(table: &[u8], offset: usize) -> Result<&str, SimError> {
    let tail = table
        .get(offset..)
        .ok_or_else(|| SimError::Parse("ELF string offset is invalid".to_string()))?;
    let end = tail
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| SimError::Parse("ELF string is unterminated".to_string()))?;
    std::str::from_utf8(&tail[..end])
        .map_err(|_| SimError::Parse("ELF string is not UTF-8".to_string()))
}

pub(super) fn read_u16(data: &[u8], offset: usize) -> Result<u16, SimError> {
    let bytes = slice(data, offset, 2)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

pub(super) fn read_u32(data: &[u8], offset: usize) -> Result<u32, SimError> {
    let bytes = slice(data, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub(super) fn slice(data: &[u8], offset: usize, length: usize) -> Result<&[u8], SimError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| SimError::Parse("ELF offset overflow".to_string()))?;
    data.get(offset..end)
        .ok_or_else(|| SimError::Parse("ELF offset is out of bounds".to_string()))
}

pub(super) fn check_table(
    data: &[u8],
    offset: u32,
    count: u16,
    size: usize,
) -> Result<(), SimError> {
    let length = usize::from(count)
        .checked_mul(size)
        .ok_or_else(|| SimError::Parse("ELF table overflow".to_string()))?;
    slice(data, offset as usize, length).map(|_| ())
}
