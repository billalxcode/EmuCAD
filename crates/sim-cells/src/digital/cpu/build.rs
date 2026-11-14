use super::*;

pub(super) fn instruction_inputs(builder: &mut DigitalBuilder) -> Vec<NetId> {
    (0..16)
        .map(|index| builder.input(&format!("instr{index}")))
        .collect()
}

pub(super) fn build_circuit(
    builder: &mut DigitalBuilder,
    instruction: &[NetId],
) -> Result<BuiltCpu, SimError> {
    let fields = extract_fields(instruction)?;
    let phase = builder.net();
    let decode = build_decode(builder, fields, phase)?;
    let (registers, alu, write_data) = build_datapath(builder, &decode)?;
    connect_write_data(builder, &alu.result, &write_data);
    let zero = latch_zero(builder, alu.zero, decode.signals.flag_write);
    let branch = build_branch(builder, &decode, zero);
    let (pc, ir, halted) = build_state(builder, &decode, instruction, branch)?;
    Ok(BuiltCpu {
        registers,
        pc,
        ir,
        halted,
    })
}

pub(super) fn build_state(
    builder: &mut DigitalBuilder,
    decode: &Decode,
    instruction: &[NetId],
    branch: Branch,
) -> Result<(ProgramCounter, Vec<NetId>, NetId), SimError> {
    let target = extended_target(builder, &decode.fields.target);
    let pc = ProgramCounter::new(builder, 16, branch.pc_load, &target, decode.fetch)?;
    let ir = register(builder, 16, instruction, decode.fetch)?;
    let halted = builder.net();
    let halt_next = or_all(builder, &[halted, branch.halt]);
    builder.dff_into(halt_next, halted);
    builder.dff_into(decode.fetch, decode.execute);
    Ok((pc, ir, halted))
}

pub(super) fn extract_fields(instruction: &[NetId]) -> Result<Fields, SimError> {
    if instruction.len() != 16 {
        return Err(SimError::InvalidNetlist(
            "instruction must be 16 bits".to_string(),
        ));
    }
    Ok(Fields {
        opcode: instruction[11..16].to_vec(),
        rd: instruction[6..11].to_vec(),
        rs: instruction[0..5].to_vec(),
        imm: instruction[0..6].to_vec(),
        target: instruction[0..11].to_vec(),
    })
}

pub(super) fn build_decode(
    builder: &mut DigitalBuilder,
    fields: Fields,
    phase: NetId,
) -> Result<Decode, SimError> {
    let lines = decoder(builder, &fields.opcode)?;
    let fetch = builder.not(phase);
    let signals = decode_signals(builder, &lines, phase);
    Ok(Decode {
        fields,
        lines,
        fetch,
        execute: phase,
        signals,
    })
}

pub(super) fn decode_signals(
    builder: &mut DigitalBuilder,
    lines: &[NetId],
    phase: NetId,
) -> DecodeSignals {
    DecodeSignals {
        reg_write: write_enable(builder, lines, phase),
        flag_write: flag_enable(builder, lines, phase),
        use_imm: lines[OP_LDI as usize],
        alu_op: alu_op(builder, lines),
    }
}

pub(super) fn write_enable(builder: &mut DigitalBuilder, lines: &[NetId], execute: NetId) -> NetId {
    let terms: Vec<NetId> = WRITE_OPCODES.iter().map(|&op| lines[op]).collect();
    let any = or_all(builder, &terms);
    builder.and(execute, any)
}

pub(super) fn flag_enable(builder: &mut DigitalBuilder, lines: &[NetId], execute: NetId) -> NetId {
    let terms: Vec<NetId> = FLAG_OPCODES.iter().map(|&op| lines[op]).collect();
    let any = or_all(builder, &terms);
    builder.and(execute, any)
}

pub(super) fn alu_op(builder: &mut DigitalBuilder, lines: &[NetId]) -> Vec<NetId> {
    vec![
        encoded_signal(builder, lines, &ALU_BIT0),
        encoded_signal(builder, lines, &ALU_BIT1),
        encoded_signal(builder, lines, &ALU_BIT2),
    ]
}

pub(super) fn encoded_signal(
    builder: &mut DigitalBuilder,
    lines: &[NetId],
    opcodes: &[usize],
) -> NetId {
    let terms: Vec<NetId> = opcodes.iter().map(|&op| lines[op]).collect();
    or_all(builder, &terms)
}

pub(super) fn build_datapath(
    builder: &mut DigitalBuilder,
    decode: &Decode,
) -> Result<(RegisterFile, DigitalAlu, Vec<NetId>), SimError> {
    let write_data: Vec<NetId> = (0..8).map(|_| builder.net()).collect();
    let ports = register_ports(decode, write_data.clone());
    let registers = register_file(builder, 32, 8, &ports)?;
    let second = effective_second(
        builder,
        registers.read_b(),
        &decode.fields,
        decode.signals.use_imm,
    );
    let alu = digital_alu(builder, registers.read_a(), &second, &decode.signals.alu_op)?;
    Ok((registers, alu, write_data))
}

pub(super) fn register_ports(decode: &Decode, write_data: Vec<NetId>) -> RegisterPorts {
    RegisterPorts {
        write_address: decode.fields.rd.clone(),
        write_data,
        write_enable: decode.signals.reg_write,
        read_address_a: decode.fields.rd.clone(),
        read_address_b: decode.fields.rs.clone(),
    }
}

pub(super) fn effective_second(
    builder: &mut DigitalBuilder,
    register_second: &[NetId],
    fields: &Fields,
    use_imm: NetId,
) -> Vec<NetId> {
    let zero = builder.constant(false);
    let mut values = Vec::with_capacity(8);
    for (index, &value) in register_second.iter().enumerate().take(5) {
        values.push(builder.mux(value, fields.imm[index], use_imm));
    }
    values.push(builder.mux(zero, fields.imm[5], use_imm));
    values.push(zero);
    values.push(zero);
    values
}

pub(super) fn connect_write_data(builder: &mut DigitalBuilder, source: &[NetId], target: &[NetId]) {
    for (input, output) in source.iter().zip(target) {
        builder.buf_into(*input, *output);
    }
}

pub(super) fn latch_zero(builder: &mut DigitalBuilder, value: NetId, enable: NetId) -> NetId {
    let zero = builder.net();
    let next = builder.mux(zero, value, enable);
    builder.dff_into(next, zero);
    zero
}

pub(super) fn build_branch(builder: &mut DigitalBuilder, decode: &Decode, zero: NetId) -> Branch {
    let not_zero = builder.not(zero);
    let jz_taken = builder.and(decode.lines[OP_JZ as usize], zero);
    let jnz_taken = builder.and(decode.lines[OP_JNZ as usize], not_zero);
    let any = or_all(
        builder,
        &[decode.lines[OP_JMP as usize], jz_taken, jnz_taken],
    );
    let pc_load = builder.and(decode.execute, any);
    let halt = builder.and(decode.execute, decode.lines[OP_HLT as usize]);
    Branch { pc_load, halt }
}

pub(super) fn extended_target(builder: &mut DigitalBuilder, target: &[NetId]) -> Vec<NetId> {
    let zero = builder.constant(false);
    let mut value = target.to_vec();
    value.resize(16, zero);
    value
}
