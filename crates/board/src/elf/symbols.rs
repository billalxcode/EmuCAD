use super::*;

pub(super) fn parse_symbols(data: &[u8], header: ElfHeader) -> Result<Vec<ElfSymbol>, SimError> {
    let sections = parse_sections(data, header)?;
    let mut symbols = Vec::new();
    for section in &sections {
        if section.kind == SHT_SYMTAB {
            symbols.extend(parse_symbol_table(data, section, &sections)?);
        }
    }
    Ok(symbols)
}

fn parse_symbol_table(
    data: &[u8],
    section: &Section,
    sections: &[Section],
) -> Result<Vec<ElfSymbol>, SimError> {
    let names = sections
        .get(section.link as usize)
        .ok_or_else(|| SimError::Parse("ELF symtab link is invalid".to_string()))?;
    let string_table = slice(data, names.offset as usize, names.size as usize)?;
    let entry_size = symbol_entry_size(section);
    let mut symbols = Vec::new();
    for entry in (0..section.size as usize).step_by(entry_size) {
        let offset = section.offset as usize + entry;
        append_symbol(&mut symbols, data, offset, string_table)?;
    }
    Ok(symbols)
}

fn symbol_entry_size(section: &Section) -> usize {
    if section.entry_size == 0 {
        SYMBOL_SIZE
    } else {
        section.entry_size as usize
    }
}

fn parse_sections(data: &[u8], header: ElfHeader) -> Result<Vec<Section>, SimError> {
    let mut sections = Vec::new();
    for index in 0..usize::from(header.section_count) {
        let offset = header.section_offset as usize + index * SECTION_HEADER_SIZE;
        sections.push(Section {
            kind: read_u32(data, offset + 4)?,
            offset: read_u32(data, offset + 16)?,
            size: read_u32(data, offset + 20)?,
            link: read_u32(data, offset + 24)?,
            entry_size: read_u32(data, offset + 36)?,
        });
    }
    Ok(sections)
}
fn append_symbol(
    symbols: &mut Vec<ElfSymbol>,
    data: &[u8],
    offset: usize,
    names: &[u8],
) -> Result<(), SimError> {
    let (name_offset, address, size, info, section) = symbol_fields(data, offset)?;
    if section == 0 {
        return Ok(());
    }
    let name = read_string(names, name_offset)?;
    if name.is_empty() {
        return Ok(());
    }
    symbols.push(make_symbol(name.to_string(), address, size, info));
    Ok(())
}

fn make_symbol(name: String, address: u32, size: u32, info: u8) -> ElfSymbol {
    ElfSymbol {
        name,
        address,
        size,
        kind: symbol_kind(info),
    }
}

fn symbol_fields(data: &[u8], offset: usize) -> Result<(usize, u32, u32, u8, u16), SimError> {
    let name_offset = read_u32(data, offset)? as usize;
    let address = read_u32(data, offset + 4)?;
    let size = read_u32(data, offset + 8)?;
    let info = data
        .get(offset + 12)
        .copied()
        .ok_or_else(|| SimError::Parse("ELF symbol is truncated".to_string()))?;
    let section = read_u16(data, offset + 14)?;
    Ok((name_offset, address, size, info, section))
}

fn symbol_kind(info: u8) -> ElfSymbolKind {
    match info & 0x0F {
        1 => ElfSymbolKind::Object,
        2 => ElfSymbolKind::Function,
        _ => ElfSymbolKind::Other,
    }
}
