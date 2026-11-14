use super::{Bootloader, Frame, FrameError, parse_frame};

fn frame(command: u8, sequence: u8, data: &[u8]) -> Vec<u8> {
    let complement = !sequence;
    let mut bytes = vec![0x1B, command, sequence, complement];
    bytes.extend_from_slice(data);
    let checksum = data
        .iter()
        .fold(command ^ sequence ^ complement, |value, byte| value ^ byte);
    bytes.push(checksum);
    bytes
}

fn parse(bytes: &[u8]) -> Frame {
    let mut buffer = bytes.to_vec();
    parse_frame(&mut buffer).expect("parse").expect("frame")
}

#[test]
fn parses_get_sync() {
    let frame = parse(&frame(0x30, 1, &[]));
    assert_eq!(frame.command, 0x30);
    assert_eq!(frame.sequence, 1);
    assert!(frame.data.is_empty());
}

#[test]
fn rejects_bad_checksum() {
    let mut bytes = frame(0x30, 1, &[]);
    *bytes.last_mut().expect("checksum") ^= 0xFF;
    let mut buffer = bytes;
    assert_eq!(parse_frame(&mut buffer), Err(FrameError::InvalidChecksum));
}

#[test]
fn parses_stk500_get_sync() {
    let mut buffer = vec![0x30, 0x20];
    let frame = parse_frame(&mut buffer).expect("parse").expect("frame");
    assert_eq!(frame.command, 0x30);
    assert_eq!(frame.sequence, 0);
    assert!(frame.data.is_empty());
}

#[test]
fn parses_stk500_load_address() {
    let mut buffer = vec![0x55, 0x34, 0x12, 0x20];
    let frame = parse_frame(&mut buffer).expect("parse").expect("frame");
    assert_eq!(frame.command, 0x55);
    assert_eq!(frame.data, vec![0x34, 0x12]);
}

#[test]
fn parses_stk500_prog_page() {
    let mut buffer = vec![0x64, 0x00, 0x02, b'F', 0x34, 0x12, 0x20];
    let frame = parse_frame(&mut buffer).expect("parse").expect("frame");
    assert_eq!(frame.command, 0x64);
    assert_eq!(frame.data, vec![0x00, 0x02, b'F', 0x34, 0x12]);
}

#[test]
fn rejects_stk500_bad_eop() {
    let mut buffer = vec![0x30, 0x21];
    assert_eq!(parse_frame(&mut buffer), Err(FrameError::InvalidChecksum));
}

#[test]
fn parses_stk500_universal() {
    let mut buffer = vec![0x56, 0xAC, 0x80, 0x00, 0x00, 0x20];
    let frame = parse_frame(&mut buffer).expect("parse").expect("frame");
    assert_eq!(frame.command, 0x56);
    assert_eq!(frame.data, vec![0xAC, 0x80, 0x00, 0x00]);
}

#[test]
fn handles_stk500_universal() {
    let mut buffer = vec![0x56, 0xAC, 0x80, 0x00, 0x00, 0x20];
    let frame = parse_frame(&mut buffer).expect("parse").expect("frame");
    let mut bootloader = Bootloader::new();
    assert_eq!(bootloader.handle(&frame), vec![0x14, 0x00, 0x10]);
}

#[test]
fn handles_read_signature() {
    let mut bootloader = Bootloader::new();
    let response = bootloader.handle(&parse(&frame(0x75, 1, &[])));
    assert_eq!(response, vec![0x14, 0x1E, 0x95, 0x0F, 0x10]);
}

#[test]
fn programs_flash_page() {
    let mut bootloader = Bootloader::new();
    let address = frame(0x55, 1, &[0x00, 0x00]);
    bootloader.handle(&parse(&address));
    let page = frame(0x64, 2, &[0x00, 0x04, b'F', 0x05, 0xE0, 0x00, 0x00]);
    bootloader.handle(&parse(&page));
    assert_eq!(bootloader.program_words(), vec![0xE005, 0x0000]);
}

#[test]
fn reads_flash_page() {
    let mut bootloader = Bootloader::new();
    bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
    let page = frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12]);
    bootloader.handle(&parse(&page));
    bootloader.handle(&parse(&frame(0x55, 3, &[0x00, 0x00])));
    let read = frame(0x74, 4, &[0x00, 0x02, b'F']);
    assert_eq!(
        bootloader.handle(&parse(&read)),
        vec![0x14, 0x34, 0x12, 0x10]
    );
}

#[test]
fn chip_erase_clears_flash() {
    let mut bootloader = Bootloader::new();
    bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
    bootloader.handle(&parse(&frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12])));
    bootloader.handle(&parse(&frame(0x52, 3, &[])));
    assert_eq!(bootloader.program_words(), vec![0xCFFF]);
}

#[test]
fn leave_program_mode_marks_running() {
    let mut bootloader = Bootloader::new();
    bootloader.handle(&parse(&frame(0x51, 1, &[])));
    assert!(bootloader.running());
    bootloader.handle(&parse(&frame(0x30, 2, &[])));
    assert!(!bootloader.running());
}

#[test]
fn trims_trailing_erased_words() {
    let mut bootloader = Bootloader::new();
    bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
    bootloader.handle(&parse(&frame(0x64, 2, &[0x00, 0x02, b'F', 0x34, 0x12])));
    assert_eq!(bootloader.program_words(), vec![0x1234]);
}

#[test]
fn round_trips_blink_program() {
    let program: [u16; 9] = [
        0xE200, 0xB904, 0xB905, 0xEF1F, 0x951A, 0xF7F1, 0xE220, 0x2702, 0xCFF9,
    ];
    let mut payload = vec![0x00, 0x12, b'F'];
    for word in program {
        payload.extend_from_slice(&word.to_le_bytes());
    }
    let mut bootloader = Bootloader::new();
    bootloader.handle(&parse(&frame(0x55, 1, &[0x00, 0x00])));
    bootloader.handle(&parse(&frame(0x64, 2, &payload)));
    assert_eq!(bootloader.program_words(), program.to_vec());
}
