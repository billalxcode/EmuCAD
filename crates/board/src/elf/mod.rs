//! Minimal ELF32 loader and symbol reader.

use kernel::SimError;

const ELF_HEADER_SIZE: usize = 52;
const PROGRAM_HEADER_SIZE: usize = 32;
const SECTION_HEADER_SIZE: usize = 40;
const SYMBOL_SIZE: usize = 16;
const MACHINE_AVR: u16 = 83;
const PT_LOAD: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const AVR_DATA_BASE: u32 = 0x0080_0000;

/// A loadable ELF segment.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfSegment {
    /// Virtual address.
    pub address: u32,
    /// File bytes.
    pub data: Vec<u8>,
    /// Memory size including zero fill.
    pub memory_size: u32,
}

/// Symbol kind derived from `st_info`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElfSymbolKind {
    /// Function symbol.
    Function,
    /// Object symbol.
    Object,
    /// Other symbol.
    Other,
}

/// A named ELF symbol.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfSymbol {
    /// Symbol name.
    pub name: String,
    /// Symbol address.
    pub address: u32,
    /// Symbol size.
    pub size: u32,
    /// Symbol kind.
    pub kind: ElfSymbolKind,
}

/// Parsed ELF32 image.
#[derive(Clone, Debug, PartialEq)]
pub struct ElfImage {
    pub(super) entry: u32,
    segments: Vec<ElfSegment>,
    symbols: Vec<ElfSymbol>,
}

#[derive(Clone, Copy)]
pub(super) struct ElfHeader {
    pub(super) entry: u32,
    pub(super) program_offset: u32,
    pub(super) section_offset: u32,
    pub(super) program_count: u16,
    pub(super) section_count: u16,
}

#[derive(Clone, Copy)]
pub(super) struct Section {
    pub(super) kind: u32,
    pub(super) offset: u32,
    pub(super) size: u32,
    pub(super) link: u32,
    pub(super) entry_size: u32,
}

impl ElfImage {
    /// Parses an ELF32 AVR image.
    pub fn parse(data: &[u8]) -> Result<Self, SimError> {
        let header = parse_header(data)?;
        let segments = parse_segments(data, header)?;
        let symbols = parse_symbols(data, header)?;
        Ok(Self {
            entry: header.entry,
            segments,
            symbols,
        })
    }

    /// Returns the entry point.
    #[must_use]
    pub const fn entry(&self) -> u32 {
        self.entry
    }

    /// Returns loadable segments.
    #[must_use]
    pub fn segments(&self) -> &[ElfSegment] {
        &self.segments
    }

    /// Returns parsed symbols.
    #[must_use]
    pub fn symbols(&self) -> &[ElfSymbol] {
        &self.symbols
    }

    /// Writes flash segments into a byte image.
    pub fn load_flash(&self, image: &mut [u8]) -> Result<(), SimError> {
        for segment in &self.segments {
            if segment.address >= AVR_DATA_BASE {
                continue;
            }
            let start = segment.address as usize;
            let end = start
                .checked_add(segment.data.len())
                .ok_or_else(|| SimError::Parse("ELF segment overflow".to_string()))?;
            if end > image.len() {
                return Err(SimError::Parse("ELF segment exceeds image".to_string()));
            }
            image[start..end].copy_from_slice(&segment.data);
        }
        Ok(())
    }

    /// Returns flash bytes as little-endian program words.
    pub fn program_words(&self) -> Result<Vec<u16>, SimError> {
        let bytes = self.flash_bytes()?;
        Ok(bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect())
    }

    /// Returns flash bytes.
    pub fn flash_bytes(&self) -> Result<Vec<u8>, SimError> {
        let size = self
            .segments
            .iter()
            .filter(|segment| segment.address < AVR_DATA_BASE)
            .map(|segment| segment.address as usize + segment.data.len())
            .max()
            .unwrap_or(0);
        let size = size + (size % 2);
        let mut bytes = vec![0u8; size];
        self.load_flash(&mut bytes)?;
        Ok(bytes)
    }

    /// Converts flash segments to Intel HEX.
    pub fn to_hex(&self) -> Result<String, SimError> {
        Ok(crate::hex::to_intel_hex(&self.flash_bytes()?))
    }
}

mod parse;
mod reader;
mod symbols;
#[cfg(test)]
mod tests;

use parse::{parse_header, parse_segments};
use reader::*;
use symbols::parse_symbols;
