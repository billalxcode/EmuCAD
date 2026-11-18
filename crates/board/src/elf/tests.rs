use super::{ElfImage, ElfSymbolKind};

fn put_u16(data: &mut [u8], offset: usize, value: u16) {
    data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn header(data: &mut [u8]) {
    data[0..4].copy_from_slice(b"\x7fELF");
    data[4] = 1;
    data[5] = 1;
    data[6] = 1;
    put_u16(data, 16, 2);
    put_u16(data, 18, 83);
    put_u32(data, 20, 1);
    put_u32(data, 28, 52);
    put_u32(data, 32, 0x200);
    put_u16(data, 42, 32);
    put_u16(data, 44, 1);
    put_u16(data, 46, 40);
    put_u16(data, 48, 4);
    put_u16(data, 50, 3);
}

fn program_header(data: &mut [u8]) {
    put_u32(data, 52, 1);
    put_u32(data, 56, 0x100);
    put_u32(data, 68, 4);
    put_u32(data, 72, 4);
    put_u32(data, 76, 5);
    data[0x100..0x104].copy_from_slice(&[1, 2, 3, 4]);
}

fn section(data: &mut [u8], index: usize, spec: (u32, u32, u32, u32, u32, u32)) {
    let base = 0x200 + index * 40;
    put_u32(data, base, spec.0);
    put_u32(data, base + 4, spec.1);
    put_u32(data, base + 16, spec.2);
    put_u32(data, base + 20, spec.3);
    put_u32(data, base + 24, spec.4);
    put_u32(data, base + 36, spec.5);
}

fn symbols(data: &mut [u8]) {
    data[0x300..0x306].copy_from_slice(b"\0main\0");
    put_u32(data, 0x330, 1);
    put_u32(data, 0x338, 4);
    data[0x33C] = 0x12;
    put_u16(data, 0x33E, 1);
    data[0x340..0x35B].copy_from_slice(b"\0.symtab\0.strtab\0.shstrtab\0");
}

fn fixture() -> Vec<u8> {
    let mut data = vec![0u8; 0x400];
    header(&mut data);
    program_header(&mut data);
    section(&mut data, 1, (1, 2, 0x320, 32, 2, 16));
    section(&mut data, 2, (9, 3, 0x300, 6, 0, 0));
    section(&mut data, 3, (17, 3, 0x340, 27, 0, 0));
    symbols(&mut data);
    data
}

#[test]
fn parses_segments_and_symbols() {
    let image = ElfImage::parse(&fixture()).expect("elf");
    assert_eq!(image.entry(), 0);
    assert_eq!(image.segments().len(), 1);
    assert_eq!(image.segments()[0].data, vec![1, 2, 3, 4]);
    assert_eq!(image.symbols().len(), 1);
    assert_eq!(image.symbols()[0].name, "main");
    assert_eq!(image.symbols()[0].kind, ElfSymbolKind::Function);
    assert_eq!(image.program_words().expect("words"), vec![0x0201, 0x0403]);
}

#[test]
fn loads_flash_segment() {
    let image = ElfImage::parse(&fixture()).expect("elf");
    let mut flash = vec![0u8; 8];
    image.load_flash(&mut flash).expect("load");
    assert_eq!(flash, vec![1, 2, 3, 4, 0, 0, 0, 0]);
}

#[test]
fn exports_intel_hex() {
    let image = ElfImage::parse(&fixture()).expect("elf");
    let hex = image.to_hex().expect("hex");
    assert!(hex.contains(":0400000001020304F2"));
    assert!(hex.ends_with(":00000001FF\n"));
}

#[test]
fn rejects_invalid_headers() {
    let mut data = fixture();
    data[0] = 0;
    assert!(ElfImage::parse(&data).is_err());
    assert!(ElfImage::parse(&[0u8; 10]).is_err());
    let mut data = fixture();
    put_u16(&mut data, 18, 1);
    assert!(ElfImage::parse(&data).is_err());
}
