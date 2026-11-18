//! Digital SPI master circuit with one tick per bit.

use crate::catalog_bits::{equals, increment};
use crate::catalog_digital::{DigitalEntry, finish};
use crate::{DigitalBuilder, NetId};

const INPUTS: [&str; 11] = [
    "reset",
    "start",
    "mosi_data0",
    "mosi_data1",
    "mosi_data2",
    "mosi_data3",
    "mosi_data4",
    "mosi_data5",
    "mosi_data6",
    "mosi_data7",
    "miso",
];
const OUTPUTS: [&str; 12] = [
    "sclk",
    "mosi",
    "cs",
    "busy",
    "miso_data0",
    "miso_data1",
    "miso_data2",
    "miso_data3",
    "miso_data4",
    "miso_data5",
    "miso_data6",
    "miso_data7",
];

#[derive(Clone, Copy)]
struct SpiInputs {
    reset: NetId,
    start: NetId,
    mosi_data: [NetId; 8],
    miso: NetId,
}

#[derive(Clone, Copy)]
struct SpiState {
    busy: NetId,
    count: [NetId; 4],
    tx_shift: [NetId; 8],
    rx_shift: [NetId; 8],
}

pub(crate) fn spi_entry() -> DigitalEntry {
    finish("mcu.serial.spi", |builder| {
        let inputs = spi_inputs(builder);
        let state = spi_state(builder);
        spi_registers(builder, inputs, state);
        spi_outputs(builder, state);
        (INPUTS.to_vec(), OUTPUTS.to_vec())
    })
}

fn spi_inputs(builder: &mut DigitalBuilder) -> SpiInputs {
    SpiInputs {
        reset: builder.input("reset"),
        start: builder.input("start"),
        mosi_data: [
            builder.input("mosi_data0"),
            builder.input("mosi_data1"),
            builder.input("mosi_data2"),
            builder.input("mosi_data3"),
            builder.input("mosi_data4"),
            builder.input("mosi_data5"),
            builder.input("mosi_data6"),
            builder.input("mosi_data7"),
        ],
        miso: builder.input("miso"),
    }
}

fn spi_state(builder: &mut DigitalBuilder) -> SpiState {
    SpiState {
        busy: builder.net(),
        count: [builder.net(), builder.net(), builder.net(), builder.net()],
        tx_shift: nets8(builder),
        rx_shift: nets8(builder),
    }
}

fn spi_registers(builder: &mut DigitalBuilder, inputs: SpiInputs, state: SpiState) {
    let not_busy = builder.not(state.busy);
    let load = builder.and(inputs.start, not_busy);
    let count8 = equals(builder, &state.count, 8);
    let not_count8 = builder.not(count8);
    let not_reset = builder.not(inputs.reset);
    let active = builder.or(state.busy, load);
    let shifting = builder.and(state.busy, not_count8);
    spi_busy(builder, state.busy, load, count8, not_reset);
    spi_count(builder, state.count, load, count8, active, not_reset);
    spi_tx(
        builder,
        state.tx_shift,
        &inputs.mosi_data,
        load,
        shifting,
        not_reset,
    );
    spi_rx(
        builder,
        state.rx_shift,
        inputs.miso,
        load,
        shifting,
        not_reset,
    );
}

fn spi_outputs(builder: &mut DigitalBuilder, state: SpiState) {
    let count8 = equals(builder, &state.count, 8);
    let not_count8 = builder.not(count8);
    let sclk = builder.and(state.busy, not_count8);
    let mosi = builder.and(state.tx_shift[7], state.busy);
    let cs = builder.not(state.busy);
    let done = builder.and(state.busy, count8);
    builder.output("sclk", sclk);
    builder.output("mosi", mosi);
    builder.output("cs", cs);
    builder.output("busy", state.busy);
    for index in 0..8 {
        builder.output(&format!("miso_data{index}"), state.rx_shift[7 - index]);
    }
    builder.output("done", done);
}

fn nets8(builder: &mut DigitalBuilder) -> [NetId; 8] {
    [
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
        builder.net(),
    ]
}

fn spi_busy(
    builder: &mut DigitalBuilder,
    busy: NetId,
    load: NetId,
    count8: NetId,
    not_reset: NetId,
) {
    let not_count8 = builder.not(count8);
    let keep = builder.and(busy, not_count8);
    let set = builder.or(load, keep);
    let next = builder.and(set, not_reset);
    builder.dff_into(next, busy);
}

fn spi_count(
    builder: &mut DigitalBuilder,
    count: [NetId; 4],
    load: NetId,
    count8: NetId,
    active: NetId,
    not_reset: NetId,
) {
    let incremented = increment(builder, &count);
    let zero = builder.constant(false);
    for index in 0..4 {
        let value = builder.mux(incremented[index], zero, count8);
        let value = builder.mux(value, zero, load);
        let value = builder.and(value, active);
        let value = builder.and(value, not_reset);
        builder.dff_into(value, count[index]);
    }
}

fn spi_tx(
    builder: &mut DigitalBuilder,
    tx_shift: [NetId; 8],
    mosi_data: &[NetId; 8],
    load: NetId,
    shifting: NetId,
    not_reset: NetId,
) {
    let zero = builder.constant(false);
    for index in 0..8 {
        let shifted = if index > 0 { tx_shift[index - 1] } else { zero };
        let value = builder.mux(tx_shift[index], shifted, shifting);
        let value = builder.mux(value, mosi_data[index], load);
        let value = builder.and(value, not_reset);
        builder.dff_into(value, tx_shift[index]);
    }
}

fn spi_rx(
    builder: &mut DigitalBuilder,
    rx_shift: [NetId; 8],
    miso: NetId,
    load: NetId,
    shifting: NetId,
    not_reset: NetId,
) {
    let zero = builder.constant(false);
    for index in 0..8 {
        let input = if index == 0 {
            miso
        } else {
            rx_shift[index - 1]
        };
        let value = builder.mux(rx_shift[index], input, shifting);
        let value = builder.mux(value, zero, load);
        let value = builder.and(value, not_reset);
        builder.dff_into(value, rx_shift[index]);
    }
}

#[cfg(test)]
mod tests {
    use super::spi_entry;

    fn entry() -> super::DigitalEntry {
        spi_entry()
    }

    fn set_byte(entry: &mut super::DigitalEntry, value: u8) {
        for index in 0..8 {
            let bit = value & (1 << index) != 0;
            entry
                .circuit
                .set_input(&format!("mosi_data{index}"), bit)
                .expect("input");
        }
    }

    fn output_byte(entry: &super::DigitalEntry) -> u8 {
        let mut value = 0u8;
        for index in 0..8 {
            if entry.circuit.output(&format!("miso_data{index}")) == Some(true) {
                value |= 1 << index;
            }
        }
        value
    }

    #[test]
    fn spi_sends_and_receives_byte() {
        let mut entry = entry();
        set_byte(&mut entry, 0xa5);
        entry.circuit.set_input("start", true).expect("input");
        entry.circuit.tick().expect("tick");
        entry.circuit.set_input("start", false).expect("input");
        let received = 0x3c;
        let mut sent = Vec::new();
        for index in 0..8 {
            sent.push(entry.circuit.output("mosi"));
            let bit = received & (1 << (7 - index)) != 0;
            entry.circuit.set_input("miso", bit).expect("input");
            entry.circuit.tick().expect("tick");
        }
        let expected = [
            Some(true),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(true),
        ];
        assert_eq!(sent, expected);
        assert_eq!(entry.circuit.output("done"), Some(true));
        assert_eq!(output_byte(&entry), received);
    }
}
