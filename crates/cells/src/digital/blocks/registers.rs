use super::*;

pub fn register(
    builder: &mut DigitalBuilder,
    width: usize,
    data: &[NetId],
    load: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 64 || data.len() != width {
        return Err(SimError::InvalidNetlist(
            "register width mismatch".to_string(),
        ));
    }
    Ok((0..width)
        .map(|bit| register_bit(builder, data[bit], load))
        .collect())
}

/// Builds one enabled storage bit.
pub fn register_bit(builder: &mut DigitalBuilder, data: NetId, load: NetId) -> NetId {
    let output = builder.net();
    let next = builder.mux(output, data, load);
    builder.dff_into(next, output);
    output
}

/// Builds a combinational incrementer.
#[must_use]
pub fn increment(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut carry = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for &bit in bits {
        output.push(builder.xor(bit, carry));
        carry = builder.and(bit, carry);
    }
    output
}

/// Builds a combinational decrementer.
#[must_use]
pub fn decrement(builder: &mut DigitalBuilder, bits: &[NetId]) -> Vec<NetId> {
    let mut borrow = builder.constant(true);
    let mut output = Vec::with_capacity(bits.len());
    for &bit in bits {
        let inverted = builder.not(bit);
        output.push(builder.xor(bit, borrow));
        borrow = builder.and(inverted, borrow);
    }
    output
}

/// Builds a combinational full adder.
#[must_use]
pub fn full_adder(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    carry_in: NetId,
) -> (NetId, NetId) {
    let partial = builder.xor(first, second);
    let sum = builder.xor(partial, carry_in);
    let lower = builder.and(first, second);
    let upper = builder.and(partial, carry_in);
    (sum, builder.or(lower, upper))
}

/// Builds a ripple counter with enable.
pub fn counter(
    builder: &mut DigitalBuilder,
    width: usize,
    enable: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 16 {
        return Err(SimError::InvalidNetlist(
            "counter width must be 1..=16".to_string(),
        ));
    }
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    for index in 0..width {
        let toggle = counter_toggle(builder, &bits, index, enable);
        let next = builder.xor(bits[index], toggle);
        builder.dff_into(next, bits[index]);
    }
    Ok(bits)
}

fn counter_toggle(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    index: usize,
    enable: NetId,
) -> NetId {
    let mut terms = Vec::with_capacity(index + 1);
    terms.push(enable);
    terms.extend_from_slice(&bits[..index]);
    and_all(builder, &terms)
}

/// Builds a shift register that shifts when enable is high.
pub fn shift_register(
    builder: &mut DigitalBuilder,
    width: usize,
    input: NetId,
    enable: NetId,
) -> Result<Vec<NetId>, SimError> {
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "shift register width must be 1..=64".to_string(),
        ));
    }
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    for index in 0..width {
        let source = shift_source(&bits, index, input);
        let next = builder.mux(bits[index], source, enable);
        builder.dff_into(next, bits[index]);
    }
    Ok(bits)
}

fn shift_source(bits: &[NetId], index: usize, input: NetId) -> NetId {
    if index == 0 { input } else { bits[index - 1] }
}
