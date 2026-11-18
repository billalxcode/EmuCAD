use super::{
    Cpu, OP_ADD, OP_HLT, OP_JNZ, OP_JZ, OP_LDI, OP_SUB, encode_alu, encode_branch, encode_hlt,
    encode_ldi,
};

#[test]
fn cpu_adds_register_values() {
    let program = vec![
        encode_ldi(1, 5),
        encode_ldi(2, 3),
        encode_alu(super::OP_MOV, 3, 1),
        encode_alu(OP_ADD, 3, 2),
        encode_hlt(),
    ];
    let mut cpu = Cpu::new(program).expect("cpu");
    let traces = cpu.run(20).expect("run");
    assert_eq!(traces.len(), 5);
    assert_eq!(cpu.register(3), Some(8), "{traces:?}");
    assert!(cpu.is_halted());
}

#[test]
fn cpu_branches_on_zero() {
    let program = vec![
        encode_ldi(1, 0),
        encode_alu(OP_SUB, 1, 1),
        encode_branch(OP_JZ, 5),
        encode_ldi(2, 1),
        encode_ldi(2, 2),
        encode_ldi(3, 7),
        encode_hlt(),
    ];
    let mut cpu = Cpu::new(program).expect("cpu");
    let traces = cpu.run(30).expect("run");
    assert_eq!(cpu.register(2), Some(0));
    assert_eq!(cpu.register(3), Some(7));
    assert_eq!(traces[2].pc, 5);
}

#[test]
fn cpu_loops_until_zero() {
    let program = vec![
        encode_ldi(1, 0),
        encode_ldi(2, 3),
        encode_ldi(3, 1),
        encode_alu(OP_ADD, 1, 3),
        encode_alu(OP_SUB, 2, 3),
        encode_branch(OP_JNZ, 3),
        encode_hlt(),
    ];
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.run(100).expect("run");
    assert_eq!(cpu.register(1), Some(3));
    assert_eq!(cpu.register(2), Some(0));
    assert!(cpu.is_halted());
}

#[test]
fn cpu_reset_clears_state() {
    let program = vec![encode_ldi(1, 9), encode_hlt()];
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.run(10).expect("run");
    cpu.reset().expect("reset");
    assert_eq!(cpu.pc(), 0);
    assert_eq!(cpu.instruction(), 0);
    assert!(!cpu.is_halted());
    assert_eq!(cpu.register(1), Some(0));
}

#[test]
fn cpu_jumps_unconditionally() {
    let program = vec![
        encode_branch(super::OP_JMP, 2),
        encode_ldi(1, 9),
        encode_ldi(2, 4),
        encode_hlt(),
    ];
    let mut cpu = Cpu::new(program).expect("cpu");
    cpu.run(20).expect("run");
    assert_eq!(cpu.register(1), Some(0));
    assert_eq!(cpu.register(2), Some(4));
}

fn trace_cpu() -> Cpu {
    let program = vec![
        encode_ldi(1, 1),
        encode_ldi(2, 2),
        encode_alu(OP_ADD, 1, 2),
        encode_hlt(),
    ];
    Cpu::new(program).expect("cpu")
}

#[test]
fn cpu_trace_matches_expected_sequence() {
    let mut cpu = trace_cpu();
    let traces = cpu.run(10).expect("run");
    let expected = [
        (1, encode_ldi(1, 1)),
        (2, encode_ldi(2, 2)),
        (3, encode_alu(OP_ADD, 1, 2)),
        (4, encode_hlt()),
    ];
    assert_eq!(traces.len(), expected.len());
    for (trace, (pc, instruction)) in traces.iter().zip(expected) {
        assert_eq!(trace.pc, pc);
        assert_eq!(trace.instruction, instruction);
    }
    assert_eq!(traces[2].registers[1], 3);
    assert!(traces[3].halted);
}

#[test]
fn cpu_rejects_empty_program() {
    assert!(Cpu::new(Vec::new()).is_err());
}

#[test]
fn cpu_after_halt_returns_no_trace() {
    let program = vec![encode_hlt()];
    let mut cpu = Cpu::new(program).expect("cpu");
    assert!(cpu.step().expect("step").is_some());
    assert!(cpu.step().expect("step").is_none());
}

#[test]
fn opcode_encoding_round_trips() {
    assert_eq!(encode_ldi(31, 63) >> 11, OP_LDI);
    assert_eq!(encode_alu(OP_ADD, 5, 6) & 0x7FF, (5 << 6) | 6);
    assert_eq!(encode_branch(OP_JZ, 0x7FF) & 0x7FF, 0x7FF);
    assert_eq!(encode_hlt() >> 11, OP_HLT);
}
