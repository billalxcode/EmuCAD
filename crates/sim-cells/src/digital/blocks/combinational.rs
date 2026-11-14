use super::*;

/// Reduces nets with AND. An empty list yields true.
#[must_use]
pub fn and_all(builder: &mut DigitalBuilder, inputs: &[NetId]) -> NetId {
    if inputs.is_empty() {
        return builder.constant(true);
    }
    inputs[1..]
        .iter()
        .fold(inputs[0], |left, right| builder.and(left, *right))
}

/// Reduces nets with OR. An empty list yields false.
#[must_use]
pub fn or_all(builder: &mut DigitalBuilder, inputs: &[NetId]) -> NetId {
    if inputs.is_empty() {
        return builder.constant(false);
    }
    inputs[1..]
        .iter()
        .fold(inputs[0], |left, right| builder.or(left, *right))
}

/// Builds a multiplexer tree. The number of inputs must be two to the
/// power of the number of select nets.
pub fn mux_tree(
    builder: &mut DigitalBuilder,
    inputs: &[NetId],
    select: &[NetId],
) -> Result<NetId, SimError> {
    if inputs.len() != 1 << select.len() {
        return Err(SimError::InvalidNetlist(
            "mux tree shape mismatch".to_string(),
        ));
    }
    let mut level = inputs.to_vec();
    for &bit in select {
        level = level
            .chunks(2)
            .map(|pair| builder.mux(pair[0], pair[1], bit))
            .collect();
    }
    Ok(level[0])
}

/// Builds a one-hot decoder.
pub fn decoder(builder: &mut DigitalBuilder, inputs: &[NetId]) -> Result<Vec<NetId>, SimError> {
    if inputs.len() > 8 {
        return Err(SimError::InvalidNetlist(
            "decoder supports at most 8 inputs".to_string(),
        ));
    }
    let count = 1usize << inputs.len();
    Ok((0..count)
        .map(|value| decoder_line(builder, inputs, value))
        .collect())
}

fn decoder_line(builder: &mut DigitalBuilder, inputs: &[NetId], value: usize) -> NetId {
    let terms: Vec<NetId> = inputs
        .iter()
        .enumerate()
        .map(|(index, &input)| decoder_term(builder, input, value, index))
        .collect();
    and_all(builder, &terms)
}

fn decoder_term(builder: &mut DigitalBuilder, input: NetId, value: usize, index: usize) -> NetId {
    if value & (1 << index) == 0 {
        builder.not(input)
    } else {
        input
    }
}

/// Reads a little-endian bit vector from a circuit.
#[must_use]
pub fn read_bits(circuit: &DigitalCircuit, bits: &[NetId]) -> u64 {
    let mut value = 0;
    for (index, &bit) in bits.iter().enumerate() {
        if circuit.value(bit) {
            value |= 1 << index;
        }
    }
    value
}
