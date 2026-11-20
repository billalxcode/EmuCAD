use super::*;

pub fn register(
    builder: &mut DigitalBuilder,
    width: usize,
    data: &[NetId],
    load: NetId,
) -> Result<Vec<NetId>, SimError> {
    validate_register(width, data)?;
    Ok((0..width)
        .map(|bit| register_bit(builder, data[bit], load))
        .collect())
}

/// Controls for an extended register.
pub struct RegisterControl<'a> {
    /// Load enable.
    pub load: NetId,
    /// Synchronous clear to zero.
    pub clear: NetId,
    /// Synchronous preset to all ones.
    pub preset: NetId,
    /// Optional byte write mask; an empty slice enables every byte.
    pub mask: &'a [NetId],
}

/// Builds a register with load, clear, preset, and optional byte mask.
///
/// Clear has priority over preset, which has priority over load.
pub fn register_extended(
    builder: &mut DigitalBuilder,
    width: usize,
    data: &[NetId],
    control: &RegisterControl,
) -> Result<Vec<NetId>, SimError> {
    validate_register(width, data)?;
    validate_register_mask(width, control.mask)?;
    let zero = builder.constant(false);
    let one = builder.constant(true);
    Ok((0..width)
        .map(|bit| register_control_bit(builder, data[bit], control, bit, zero, one))
        .collect())
}

fn register_control_bit(
    builder: &mut DigitalBuilder,
    data: NetId,
    control: &RegisterControl,
    bit: usize,
    zero: NetId,
    one: NetId,
) -> NetId {
    let output = builder.net();
    let enable = byte_enable(builder, control, bit);
    let loaded = builder.mux(output, data, enable);
    let preset = builder.mux(loaded, one, control.preset);
    let next = builder.mux(preset, zero, control.clear);
    builder.dff_into(next, output);
    output
}

fn byte_enable(builder: &mut DigitalBuilder, control: &RegisterControl, bit: usize) -> NetId {
    if control.mask.is_empty() {
        return control.load;
    }
    builder.and(control.load, control.mask[bit / 8])
}

fn validate_register(width: usize, data: &[NetId]) -> Result<(), SimError> {
    if width == 0 || width > 64 {
        return Err(SimError::InvalidNetlist(
            "register width must be 1..=64".to_string(),
        ));
    }
    if data.len() != width {
        return Err(SimError::InvalidNetlist(
            "register data width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_register_mask(width: usize, mask: &[NetId]) -> Result<(), SimError> {
    if mask.is_empty() {
        return Ok(());
    }
    if mask.len() != width.div_ceil(8) {
        return Err(SimError::InvalidNetlist(
            "register mask must have one bit per byte".to_string(),
        ));
    }
    Ok(())
}

/// Builds one enabled storage bit.
pub fn register_bit(builder: &mut DigitalBuilder, data: NetId, load: NetId) -> NetId {
    let output = builder.net();
    let next = builder.mux(output, data, load);
    builder.dff_into(next, output);
    output
}

/// Connects next-state values to existing storage nets.
pub fn clock_vector(builder: &mut DigitalBuilder, bits: &[NetId], next: &[NetId]) {
    for (bit, &value) in bits.iter().zip(next) {
        builder.dff_into(value, *bit);
    }
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
