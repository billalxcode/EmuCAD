use super::*;

#[test]
fn indirect_load_store_with_post_and_pre() {
    let cpu = run(vec![
        ldi(26, 0x00),
        ldi(27, 0x01),
        ldi(16, 0x55),
        0x920D | (16 << 4),
        0x900E | (17 << 4),
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0100), 0x55);
    assert_eq!(cpu.register(17), 0x55);
    assert_eq!(cpu.register(26), 0x00);
    assert_eq!(cpu.register(27), 0x01);
}

#[test]
fn ldd_and_std_displacement() {
    let cpu = run(vec![
        ldi(28, 0x00),
        ldi(29, 0x02),
        ldi(16, 0x66),
        0x8208 | (16 << 4) | 5,
        0x8008 | (17 << 4) | 5,
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0205), 0x66);
    assert_eq!(cpu.register(17), 0x66);
}

#[test]
fn lds_and_sts_direct() {
    let cpu = run(vec![
        ldi(16, 0x77),
        0x9200 | (16 << 4),
        0x0300,
        0x9000 | (17 << 4),
        0x0300,
        0x9598,
    ]);
    assert_eq!(cpu.data(0x0300), 0x77);
    assert_eq!(cpu.register(17), 0x77);
}

#[test]
fn lpm_reads_program_byte() {
    let first = ldi(30, 0);
    let cpu = run(vec![first, ldi(16, 0), 0x9104, 0x9598]);
    assert_eq!(cpu.register(16), first as u8);
}

#[test]
fn lpm_post_increment_advances_z() {
    let first = ldi(30, 0);
    let cpu = run(vec![first, 0x9105, 0x9005 | (17 << 4), 0x9598]);
    assert_eq!(cpu.register(16), first as u8);
    assert_eq!(cpu.register(17), (first >> 8) as u8);
    assert_eq!(cpu.register(30), 2);
}

#[test]
fn push_and_pop_use_stack() {
    let cpu = run(vec![
        ldi(16, 0x5A),
        0x920F | (16 << 4),
        ldi(16, 0),
        0x900F | (17 << 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(17), 0x5A);
    assert_eq!(cpu.sp(), 0x08FF);
}

#[test]
fn rcall_and_ret_return_to_caller() {
    let cpu = run(vec![
        0xD003,
        ldi(16, 1),
        0x9598,
        ldi(17, 9),
        ldi(17, 7),
        0x9508,
    ]);
    assert_eq!(cpu.register(16), 1);
    assert_eq!(cpu.register(17), 7);
    assert_eq!(cpu.sp(), 0x08FF);
}

#[test]
fn call_and_ret_use_two_word_target() {
    let cpu = run(vec![0x940E, 0x0004, ldi(16, 1), 0x9598, ldi(17, 7), 0x9508]);
    assert_eq!(cpu.register(16), 1);
    assert_eq!(cpu.register(17), 7);
}

#[test]
fn reti_sets_interrupt_flag() {
    let cpu = run(vec![0xD002, 0x9598, 0x0000, 0x9518]);
    assert!(cpu.sreg().interrupt);
}

#[test]
fn ijmp_uses_z_pointer() {
    let cpu = run(vec![
        ldi(30, 4),
        ldi(31, 0),
        0x9409,
        ldi(16, 9),
        ldi(17, 4),
        0x9598,
    ]);
    assert_eq!(cpu.register(16), 0);
    assert_eq!(cpu.register(17), 4);
}
