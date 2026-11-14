use super::*;

pub(super) fn parse_header(data: &[u8]) -> Result<ElfHeader, SimError> {
    validate_ident(data)?;
    validate_machine(data)?;
    let program_count = read_u16(data, 44)?;
    let section_count = read_u16(data, 48)?;
    validate_section_names(data, section_count)?;
    validate_tables(data, program_count, section_count)?;
    Ok(ElfHeader {
        entry: read_u32(data, 24)?,
        program_offset: read_u32(data, 28)?,
        section_offset: read_u32(data, 32)?,
        program_count,
        section_count,
    })
}

fn validate_ident(data: &[u8]) -> Result<(), SimError> {
    if data.len() < ELF_HEADER_SIZE {
        return Err(SimError::Parse("ELF header is truncated".to_string()));
    }
    if &data[0..4] != b"\x7fELF" {
        return Err(SimError::Parse("invalid ELF magic".to_string()));
    }
    if data[4] != 1 || data[5] != 1 {
        return Err(SimError::Parse(
            "ELF must be 32-bit little endian".to_string(),
        ));
    }
    Ok(())
}

fn validate_machine(data: &[u8]) -> Result<(), SimError> {
    if read_u16(data, 18)? != MACHINE_AVR {
        return Err(SimError::Parse("ELF machine is not AVR".to_string()));
    }
    Ok(())
}

fn validate_section_names(data: &[u8], count: u16) -> Result<(), SimError> {
    let index = read_u16(data, 50)?;
    if index != 0 && index >= count {
        return Err(SimError::Parse(
            "ELF section name index is invalid".to_string(),
        ));
    }
    Ok(())
}

fn validate_tables(data: &[u8], programs: u16, sections: u16) -> Result<(), SimError> {
    if programs > 0 {
        check_table(data, read_u32(data, 28)?, programs, PROGRAM_HEADER_SIZE)?;
    }
    if sections > 0 {
        check_table(data, read_u32(data, 32)?, sections, SECTION_HEADER_SIZE)?;
    }
    Ok(())
}

pub(super) fn parse_segments(data: &[u8], header: ElfHeader) -> Result<Vec<ElfSegment>, SimError> {
    let mut segments = Vec::new();
    for index in 0..usize::from(header.program_count) {
        let offset = header.program_offset as usize + index * PROGRAM_HEADER_SIZE;
        if read_u32(data, offset)? != PT_LOAD {
            continue;
        }
        let file_offset = read_u32(data, offset + 4)? as usize;
        let address = read_u32(data, offset + 8)?;
        let file_size = read_u32(data, offset + 16)? as usize;
        let memory_size = read_u32(data, offset + 20)?;
        let bytes = slice(data, file_offset, file_size)?;
        segments.push(ElfSegment {
            address,
            data: bytes.to_vec(),
            memory_size,
        });
    }
    Ok(segments)
}
