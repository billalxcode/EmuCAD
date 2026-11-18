use super::*;

pub fn hex_words(text: &str) -> Result<Vec<u16>, String> {
    let mut image = HexImage::new(FLASH_BYTES);
    image.load(text)?;
    Ok(image.words())
}

/// Reads and converts an Intel HEX file.
pub fn load_hex(path: &Path) -> Result<Vec<u16>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("read failed: {error}"))?;
    hex_words(&text)
}
