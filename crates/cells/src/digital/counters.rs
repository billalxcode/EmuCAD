//! Loadable up/down counters and generic prescalers.

use super::blocks::{and_all, decrement, increment, select_vector, validate_pair, validate_width};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Result of a generic prescaler.
pub struct Prescaler {
    /// Counter bits, least significant bit first.
    pub count: Vec<NetId>,
    /// Tick asserted for one cycle when the counter wraps.
    pub tick: NetId,
}

/// Builds a loadable up/down counter.
///
/// Load has priority over enable.
pub fn counter_loadable(
    builder: &mut DigitalBuilder,
    width: usize,
    load_value: &[NetId],
    load: NetId,
    enable: NetId,
    up_down: NetId,
) -> Result<Vec<NetId>, SimError> {
    validate_counter(width, load_value)?;
    let bits: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let incremented = increment(builder, &bits);
    let decremented = decrement(builder, &bits);
    let stepped = select_vector(builder, &decremented, &incremented, up_down)?;
    let enabled = select_vector(builder, &bits, &stepped, enable)?;
    let next = select_vector(builder, &enabled, load_value, load)?;
    clock_bits(builder, &bits, &next);
    Ok(bits)
}

/// Builds an equality match for a counter value.
pub fn counter_match(
    builder: &mut DigitalBuilder,
    bits: &[NetId],
    value: &[NetId],
) -> Result<NetId, SimError> {
    validate_pair(bits, value, "counter match", 64)?;
    let terms: Vec<NetId> = bits
        .iter()
        .zip(value)
        .map(|(&left, &right)| {
            let difference = builder.xor(left, right);
            builder.not(difference)
        })
        .collect();
    Ok(and_all(builder, &terms))
}

/// Builds a modulo-`divisor` prescaler with a one-cycle tick.
pub fn prescaler(
    builder: &mut DigitalBuilder,
    width: usize,
    divisor: u64,
    enable: NetId,
) -> Result<Prescaler, SimError> {
    validate_prescaler(width, divisor)?;
    let zero = builder.constant(false);
    let count: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let target = constant_bits(builder, divisor - 1, width);
    let terminal = counter_match(builder, &count, &target)?;
    let incremented = increment(builder, &count);
    let zeroes = vec![zero; width];
    let wrapped = select_vector(builder, &incremented, &zeroes, terminal)?;
    let next = select_vector(builder, &count, &wrapped, enable)?;
    clock_bits(builder, &count, &next);
    let tick = builder.and(enable, terminal);
    Ok(Prescaler { count, tick })
}

fn clock_bits(builder: &mut DigitalBuilder, bits: &[NetId], next: &[NetId]) {
    for (bit, &value) in bits.iter().zip(next) {
        builder.dff_into(value, *bit);
    }
}

fn constant_bits(builder: &mut DigitalBuilder, value: u64, width: usize) -> Vec<NetId> {
    (0..width)
        .map(|bit| builder.constant(value & (1 << bit) != 0))
        .collect()
}

fn validate_counter(width: usize, load_value: &[NetId]) -> Result<(), SimError> {
    validate_width(load_value, "counter load", 64)?;
    if load_value.len() != width {
        return Err(SimError::InvalidNetlist(
            "counter load width mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_prescaler(width: usize, divisor: u64) -> Result<(), SimError> {
    if width == 0 || width > 32 {
        return Err(SimError::InvalidNetlist(
            "prescaler width must be 1..=32".to_string(),
        ));
    }
    if divisor == 0 || divisor > 1u64 << width {
        return Err(SimError::InvalidNetlist(
            "prescaler divisor must be 1..=2^width".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Prescaler, counter_loadable, counter_match, prescaler};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    const WIDTH: usize = 8;

    fn counter_fixture() -> (DigitalCircuit, Vec<NetId>) {
        let mut builder = DigitalBuilder::new();
        let load_value = named_inputs(&mut builder, "v", WIDTH);
        let load = builder.input("load");
        let enable = builder.input("enable");
        let up_down = builder.input("up");
        let bits = counter_loadable(&mut builder, WIDTH, &load_value, load, enable, up_down)
            .expect("counter");
        (builder.build(), bits)
    }

    fn prescaler_fixture(divisor: u64) -> (DigitalCircuit, Prescaler) {
        let mut builder = DigitalBuilder::new();
        let enable = builder.input("enable");
        let prescaler = prescaler(&mut builder, 4, divisor, enable).expect("prescaler");
        (builder.build(), prescaler)
    }

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    fn set_counter_control(circuit: &mut DigitalCircuit, load: bool, enable: bool, up_down: bool) {
        circuit.set_input("load", load).expect("load");
        circuit.set_input("enable", enable).expect("enable");
        circuit.set_input("up", up_down).expect("up");
    }

    #[test]
    fn loadable_counter_loads_counts_and_holds() {
        let (mut circuit, bits) = counter_fixture();
        set_bits(&mut circuit, "v", 0x42, WIDTH);
        set_counter_control(&mut circuit, true, false, true);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0x42);
        set_counter_control(&mut circuit, false, true, true);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0x43);
        set_counter_control(&mut circuit, false, true, false);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0x42);
        set_counter_control(&mut circuit, false, false, true);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0x42);
    }

    #[test]
    fn loadable_counter_wraps_and_load_wins() {
        let (mut circuit, bits) = counter_fixture();
        set_bits(&mut circuit, "v", 0xFF, WIDTH);
        set_counter_control(&mut circuit, true, true, true);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0xFF);
        set_counter_control(&mut circuit, false, true, true);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0);
        set_counter_control(&mut circuit, false, true, false);
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &bits), 0xFF);
    }

    #[test]
    fn counter_match_detects_equal_value() {
        let mut builder = DigitalBuilder::new();
        let bits = named_inputs(&mut builder, "a", 4);
        let value = named_inputs(&mut builder, "b", 4);
        let matched = counter_match(&mut builder, &bits, &value).expect("match");
        let mut circuit = builder.build();
        for left in 0..16 {
            for right in 0..16 {
                set_bits(&mut circuit, "a", left, 4);
                set_bits(&mut circuit, "b", right, 4);
                circuit.settle().expect("settle");
                assert_eq!(circuit.value(matched), left == right);
            }
        }
    }

    #[test]
    fn prescaler_ticks_after_divisor_cycles() {
        let (mut circuit, prescaler) = prescaler_fixture(5);
        circuit.set_input("enable", true).expect("enable");
        for _ in 0..4 {
            circuit.tick().expect("tick");
        }
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &prescaler.count), 4);
        assert!(circuit.value(prescaler.tick));
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
        assert_eq!(read_bits(&circuit, &prescaler.count), 0);
        assert!(!circuit.value(prescaler.tick));
    }

    #[test]
    fn prescaler_holds_without_enable() {
        let (mut circuit, prescaler) = prescaler_fixture(3);
        circuit.set_input("enable", false).expect("enable");
        for _ in 0..5 {
            circuit.tick().expect("tick");
        }
        assert_eq!(read_bits(&circuit, &prescaler.count), 0);
        assert!(!circuit.value(prescaler.tick));
    }

    #[test]
    fn prescaler_rejects_invalid_divisor() {
        let mut builder = DigitalBuilder::new();
        let enable = builder.input("enable");
        assert!(prescaler(&mut builder, 4, 0, enable).is_err());
        assert!(prescaler(&mut builder, 4, 17, enable).is_err());
    }
}
