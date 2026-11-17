//! Digital UART circuit with one tick per bit.

use crate::catalog_bits::{equals4, increment4};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 11] = [
    "reset", "start", "din0", "din1", "din2", "din3", "din4", "din5", "din6", "din7", "rx",
];
const OUTPUTS: [&str; 11] = [
    "tx", "busy", "dout0", "dout1", "dout2", "dout3", "dout4", "dout5", "dout6", "dout7", "ready",
];

pub(crate) fn uart_entry() -> DigitalEntry {
    finish("mcu.serial.uart", |builder| {
        let reset = builder.input("reset");
        let start = builder.input("start");
        let din = [
            builder.input("din0"),
            builder.input("din1"),
            builder.input("din2"),
            builder.input("din3"),
            builder.input("din4"),
            builder.input("din5"),
            builder.input("din6"),
            builder.input("din7"),
        ];
        let rx = builder.input("rx");
        let (tx, busy) = tx_state(builder, reset, start, &din);
        let (dout, ready) = rx_state(builder, reset, rx);
        builder.output("tx", tx);
        builder.output("busy", busy);
        for (index, net) in dout.iter().enumerate() {
            builder.output(&format!("dout{index}"), *net);
        }
        builder.output("ready", ready);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn tx_state(
    builder: &mut DigitalBuilder,
    reset: NetId,
    start: NetId,
    din: &[NetId; 8],
) -> (NetId, NetId) {
    let busy = builder.net();
    let phase = [builder.net(), builder.net(), builder.net(), builder.net()];
    let shift = [
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
    ];
    let not_busy = builder.not(busy);
    let load = builder.and(start, not_busy);
    let busy_next = tx_busy_next(builder, reset, busy, &phase, load);
    builder.dff_into(busy_next, busy);
    let phase_next = tx_phase_next(builder, reset, busy, &phase, load);
    for (index, net) in phase.iter().enumerate() {
        builder.dff_into(phase_next[index], *net);
    }
    let shift_next = tx_shift_next(builder, reset, busy, &phase, &shift, din, load);
    for (index, net) in shift.iter().enumerate() {
        builder.dff_into(shift_next[index], *net);
    }
    let tx = tx_output(builder, &phase, &shift, busy);
    (tx, busy)
}

fn tx_busy_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    busy: NetId,
    phase: &[NetId; 4],
    load: NetId,
) -> NetId {
    let phase10 = equals4(builder, phase, 10);
    let not_phase10 = builder.not(phase10);
    let keep = builder.and(busy, not_phase10);
    let set = builder.or(load, keep);
    let not_reset = builder.not(reset);
    builder.and(set, not_reset)
}

fn tx_phase_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    busy: NetId,
    phase: &[NetId; 4],
    load: NetId,
) -> [NetId; 4] {
    let incremented = increment4(builder, phase);
    let phase10 = equals4(builder, phase, 10);
    let zero = builder.constant(false);
    let active = builder.or(busy, load);
    let not_reset = builder.not(reset);
    let mut next = [zero; 4];
    for index in 0..4 {
        let value = builder.mux(incremented[index], zero, phase10);
        let value = builder.mux(value, zero, load);
        let value = builder.and(value, active);
        next[index] = builder.and(value, not_reset);
    }
    next
}

fn tx_shift_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    busy: NetId,
    phase: &[NetId; 4],
    shift: &[NetId; 8],
    din: &[NetId; 8],
    load: NetId,
) -> [NetId; 8] {
    let zero = builder.constant(false);
    let one = builder.constant(true);
    let not_reset = builder.not(reset);
    let phase0 = equals4(builder, phase, 0);
    let phase9 = equals4(builder, phase, 9);
    let phase10 = equals4(builder, phase, 10);
    let not_phase0 = builder.not(phase0);
    let not_phase9 = builder.not(phase9);
    let not_phase10 = builder.not(phase10);
    let middle = builder.and(not_phase9, not_phase10);
    let data_phase = builder.and(not_phase0, middle);
    let shifting = builder.and(busy, data_phase);
    let mut next = [zero; 8];
    for index in 0..8 {
        let shifted = if index < 7 { shift[index + 1] } else { one };
        let value = builder.mux(shift[index], shifted, shifting);
        let value = builder.mux(value, din[index], load);
        next[index] = builder.and(value, not_reset);
    }
    next
}

fn tx_output(
    builder: &mut DigitalBuilder,
    phase: &[NetId; 4],
    shift: &[NetId; 8],
    busy: NetId,
) -> NetId {
    let one = builder.constant(true);
    let zero = builder.constant(false);
    let phase0 = equals4(builder, phase, 0);
    let phase9 = equals4(builder, phase, 9);
    let phase10 = equals4(builder, phase, 10);
    let not_phase0 = builder.not(phase0);
    let not_phase9 = builder.not(phase9);
    let not_phase10 = builder.not(phase10);
    let middle = builder.and(not_phase9, not_phase10);
    let data_phase = builder.and(not_phase0, middle);
    let data_phase = builder.and(data_phase, busy);
    let start = builder.and(phase0, busy);
    let value = builder.mux(one, shift[0], data_phase);
    builder.mux(value, zero, start)
}

fn rx_state(builder: &mut DigitalBuilder, reset: NetId, rx: NetId) -> ([NetId; 8], NetId) {
    let active = builder.net();
    let phase = [builder.net(), builder.net(), builder.net(), builder.net()];
    let shift = [
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
    ];
    let not_rx = builder.not(rx);
    let not_active = builder.not(active);
    let start_detect = builder.and(not_active, not_rx);
    let phase9 = equals4(builder, &phase, 9);
    let active_next = rx_active_next(builder, reset, active, phase9, start_detect);
    builder.dff_into(active_next, active);
    let phase_next = rx_phase_next(builder, reset, active, &phase, phase9, start_detect);
    for (index, net) in phase.iter().enumerate() {
        builder.dff_into(phase_next[index], *net);
    }
    let shift_next = rx_shift_next(builder, reset, active, &phase, &shift, rx, phase9);
    for (index, net) in shift.iter().enumerate() {
        builder.dff_into(shift_next[index], *net);
    }
    let mut dout = [shift[7]; 8];
    for index in 0..8 {
        dout[index] = shift[7 - index];
    }
    let ready = builder.and(active, phase9);
    (dout, ready)
}

fn rx_active_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    active: NetId,
    phase9: NetId,
    start_detect: NetId,
) -> NetId {
    let not_phase9 = builder.not(phase9);
    let keep = builder.and(active, not_phase9);
    let set = builder.or(start_detect, keep);
    let not_reset = builder.not(reset);
    builder.and(set, not_reset)
}

fn rx_phase_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    active: NetId,
    phase: &[NetId; 4],
    phase9: NetId,
    start_detect: NetId,
) -> [NetId; 4] {
    let incremented = increment4(builder, phase);
    let zero = builder.constant(false);
    let one = builder.constant(true);
    let not_reset = builder.not(reset);
    let active_or_start = builder.or(active, start_detect);
    let mut next = [zero; 4];
    for index in 0..4 {
        let value = builder.mux(incremented[index], zero, phase9);
        let start_bit = if index == 0 { one } else { zero };
        let value = builder.mux(value, start_bit, start_detect);
        let value = builder.and(value, active_or_start);
        next[index] = builder.and(value, not_reset);
    }
    next
}

fn rx_shift_next(
    builder: &mut DigitalBuilder,
    reset: NetId,
    active: NetId,
    phase: &[NetId; 4],
    shift: &[NetId; 8],
    rx: NetId,
    phase9: NetId,
) -> [NetId; 8] {
    let zero = builder.constant(false);
    let not_reset = builder.not(reset);
    let phase0 = equals4(builder, phase, 0);
    let not_phase0 = builder.not(phase0);
    let not_phase9 = builder.not(phase9);
    let middle = builder.and(not_phase0, not_phase9);
    let sampling = builder.and(active, middle);
    let mut next = [zero; 8];
    for index in 0..8 {
        let input = if index == 0 { rx } else { shift[index - 1] };
        let value = builder.mux(shift[index], input, sampling);
        next[index] = builder.and(value, not_reset);
    }
    next
}

#[cfg(test)]
mod tests {
    use super::uart_entry;

    fn entry() -> super::DigitalEntry {
        uart_entry()
    }

    fn set_byte(entry: &mut super::DigitalEntry, value: u8) {
        for index in 0..8 {
            let bit = value & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("din{index}"), bit)
                .expect("input");
        }
    }

    fn output_byte(entry: &super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..8 {
            if entry.circuit.output(&format!("dout{index}")) == Some(true) {
                value |= 1 << index;
            }
        }
        value
    }

    #[test]
    fn uart_transmits_start_data_and_stop() {
        let mut entry = entry();
        set_byte(&mut entry, 0xa5);
        entry.circuit.set_input("start", true).expect("input");
        entry.circuit.tick().expect("tick");
        entry.circuit.set_input("start", false).expect("input");
        let mut bits = Vec::new();
        for _ in 0..10 {
            bits.push(entry.circuit.output("tx"));
            entry.circuit.tick().expect("tick");
        }
        let expected = [
            Some(false),
            Some(true),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(true),
            Some(true),
        ];
        assert_eq!(bits, expected);
    }

    #[test]
    fn uart_receives_byte() {
        let mut entry = entry();
        entry.circuit.set_input("rx", false).expect("input");
        entry.circuit.tick().expect("tick");
        for index in 0..8 {
            let bit = 0xa5 & (1 << index) != 0;
            entry.circuit.set_input("rx", bit).expect("input");
            entry.circuit.tick().expect("tick");
        }
        entry.circuit.set_input("rx", true).expect("input");
        assert_eq!(entry.circuit.output("ready"), Some(true));
        assert_eq!(output_byte(&entry), 0xa5);
    }
}
