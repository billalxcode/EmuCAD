//! One-entry valid/ready elastic buffer.

use super::blocks::clock_vector;
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Ports of a one-entry elastic buffer.
pub struct Handshake {
    /// Registered data, least significant bit first.
    pub data_out: Vec<NetId>,
    /// Output data is valid.
    pub valid_out: NetId,
    /// Input can be accepted.
    pub ready_in: NetId,
}

/// Builds a one-entry valid/ready buffer.
pub fn handshake_register(
    builder: &mut DigitalBuilder,
    width: usize,
    data_in: &[NetId],
    valid_in: NetId,
    ready_out: NetId,
) -> Result<Handshake, SimError> {
    validate_handshake(width, data_in)?;
    let data: Vec<NetId> = (0..width).map(|_| builder.net()).collect();
    let valid = builder.net();
    let control = handshake_control(builder, valid, valid_in, ready_out);
    let next_data = handshake_data(builder, &data, data_in, control.load);
    builder.dff_into(control.next_valid, valid);
    clock_vector(builder, &data, &next_data);
    Ok(Handshake {
        data_out: data,
        valid_out: valid,
        ready_in: control.ready_in,
    })
}

struct HandshakeControl {
    ready_in: NetId,
    load: NetId,
    next_valid: NetId,
}

fn handshake_control(
    builder: &mut DigitalBuilder,
    valid: NetId,
    valid_in: NetId,
    ready_out: NetId,
) -> HandshakeControl {
    let not_valid = builder.not(valid);
    let ready_in = builder.or(not_valid, ready_out);
    let load = builder.and(valid_in, ready_in);
    let not_ready = builder.not(ready_out);
    let hold = builder.and(valid, not_ready);
    HandshakeControl {
        ready_in,
        load,
        next_valid: builder.or(load, hold),
    }
}

fn handshake_data(
    builder: &mut DigitalBuilder,
    data: &[NetId],
    data_in: &[NetId],
    load: NetId,
) -> Vec<NetId> {
    (0..data.len())
        .map(|bit| builder.mux(data[bit], data_in[bit], load))
        .collect()
}

fn validate_handshake(width: usize, data_in: &[NetId]) -> Result<(), SimError> {
    if width == 0 || width > 64 || data_in.len() != width {
        return Err(invalid("handshake width must be 1..=64"));
    }
    Ok(())
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{Handshake, handshake_register};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    const WIDTH: usize = 8;

    fn fixture() -> (DigitalCircuit, Handshake) {
        let mut builder = DigitalBuilder::new();
        let data_in = named_inputs(&mut builder, "d", WIDTH);
        let valid_in = builder.input("vi");
        let ready_out = builder.input("ro");
        let buffer = handshake_register(&mut builder, WIDTH, &data_in, valid_in, ready_out)
            .expect("handshake");
        (builder.build(), buffer)
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

    fn drive(circuit: &mut DigitalCircuit, data: u64, valid: bool, ready: bool) {
        set_bits(circuit, "d", data, WIDTH);
        circuit.set_input("vi", valid).expect("vi");
        circuit.set_input("ro", ready).expect("ro");
        circuit.settle().expect("settle");
    }

    fn tick(circuit: &mut DigitalCircuit) {
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
    }

    #[test]
    fn empty_buffer_is_ready() {
        let (mut circuit, buffer) = fixture();
        drive(&mut circuit, 0, false, false);
        assert!(circuit.value(buffer.ready_in));
        assert!(!circuit.value(buffer.valid_out));
    }

    #[test]
    fn buffer_loads_and_holds() {
        let (mut circuit, buffer) = fixture();
        drive(&mut circuit, 0x42, true, false);
        tick(&mut circuit);
        assert!(circuit.value(buffer.valid_out));
        assert!(!circuit.value(buffer.ready_in));
        assert_eq!(read_bits(&circuit, &buffer.data_out), 0x42);
        drive(&mut circuit, 0x99, true, false);
        tick(&mut circuit);
        assert_eq!(read_bits(&circuit, &buffer.data_out), 0x42);
    }

    #[test]
    fn buffer_consumes_and_reloads() {
        let (mut circuit, buffer) = fixture();
        drive(&mut circuit, 0x11, true, true);
        tick(&mut circuit);
        assert!(circuit.value(buffer.valid_out));
        assert_eq!(read_bits(&circuit, &buffer.data_out), 0x11);
        drive(&mut circuit, 0x22, true, true);
        tick(&mut circuit);
        assert_eq!(read_bits(&circuit, &buffer.data_out), 0x22);
    }

    #[test]
    fn buffer_clears_after_consume() {
        let (mut circuit, buffer) = fixture();
        drive(&mut circuit, 0x33, true, false);
        tick(&mut circuit);
        drive(&mut circuit, 0, false, true);
        tick(&mut circuit);
        assert!(!circuit.value(buffer.valid_out));
        assert!(circuit.value(buffer.ready_in));
    }

    #[test]
    fn buffer_rejects_bad_width() {
        let mut builder = DigitalBuilder::new();
        let data = named_inputs(&mut builder, "d", WIDTH);
        let valid = builder.input("vi");
        let ready = builder.input("ro");
        assert!(handshake_register(&mut builder, 4, &data, valid, ready).is_err());
    }
}
