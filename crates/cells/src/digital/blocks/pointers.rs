use super::*;

/// A loadable program counter with increment.
pub struct ProgramCounter {
    bits: Vec<NetId>,
}

impl ProgramCounter {
    /// Builds a program counter.
    pub fn new(
        builder: &mut DigitalBuilder,
        width: usize,
        load: NetId,
        target: &[NetId],
        increment_enable: NetId,
    ) -> Result<Self, SimError> {
        validate_word_width(width, target, 16)?;
        Ok(Self {
            bits: build_pc(builder, width, load, target, increment_enable),
        })
    }

    /// Returns counter bit nets.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Reads the counter value.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u64 {
        read_bits(circuit, &self.bits)
    }
}

/// A stack pointer with load, increment, and decrement.
pub struct StackPointer {
    bits: Vec<NetId>,
}

impl StackPointer {
    /// Builds a stack pointer.
    pub fn new(
        builder: &mut DigitalBuilder,
        width: usize,
        load: NetId,
        target: &[NetId],
        increment_enable: NetId,
        decrement_enable: NetId,
    ) -> Result<Self, SimError> {
        validate_word_width(width, target, 16)?;
        let controls = SpControl {
            load,
            increment: increment_enable,
            decrement: decrement_enable,
        };
        Ok(Self {
            bits: build_sp(builder, width, target, controls),
        })
    }

    /// Returns pointer bit nets.
    #[must_use]
    pub fn bits(&self) -> &[NetId] {
        &self.bits
    }

    /// Reads the pointer value.
    #[must_use]
    pub fn read(&self, circuit: &DigitalCircuit) -> u64 {
        read_bits(circuit, &self.bits)
    }
}

#[derive(Clone, Copy)]
struct SpControl {
    load: NetId,
    increment: NetId,
    decrement: NetId,
}

fn validate_word_width(width: usize, target: &[NetId], max: usize) -> Result<(), SimError> {
    if width == 0 || width > max || target.len() != width {
        return Err(SimError::InvalidNetlist("word width mismatch".to_string()));
    }
    Ok(())
}

fn build_pc(
    builder: &mut DigitalBuilder,
    width: usize,
    load: NetId,
    target: &[NetId],
    increment_enable: NetId,
) -> Vec<NetId> {
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let incremented = increment(builder, &bits);
    for index in 0..width {
        let held = builder.mux(bits[index], incremented[index], increment_enable);
        let next = builder.mux(held, target[index], load);
        builder.dff_into(next, bits[index]);
    }
    bits
}

fn build_sp(
    builder: &mut DigitalBuilder,
    width: usize,
    target: &[NetId],
    controls: SpControl,
) -> Vec<NetId> {
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    wire_sp(builder, &bits, target, controls);
    bits
}

fn wire_sp(builder: &mut DigitalBuilder, bits: &[NetId], target: &[NetId], controls: SpControl) {
    let incremented = increment(builder, bits);
    let decremented = decrement(builder, bits);
    for index in 0..bits.len() {
        let next = sp_next(
            builder,
            bits,
            &incremented,
            &decremented,
            target,
            index,
            controls,
        );
        builder.dff_into(next, bits[index]);
    }
}

fn sp_next(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    incremented: &[NetId],
    decremented: &[NetId],
    target: &[NetId],
    index: usize,
    controls: SpControl,
) -> NetId {
    let held = builder.mux(bits[index], decremented[index], controls.decrement);
    let stepped = builder.mux(held, incremented[index], controls.increment);
    builder.mux(stepped, target[index], controls.load)
}
