//! Generated board arduino component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "board.arduino.arduino_uno_board",
        name: "Arduino Uno board",
        class: ModelClass::BoardModule,
        device: None,
        pins: 8,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.arduino.breadboard_power_board",
        name: "breadboard power board",
        class: ModelClass::BoardModule,
        device: None,
        pins: 8,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.arduino.generic_mcu_board",
        name: "generic MCU board",
        class: ModelClass::BoardModule,
        device: None,
        pins: 8,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
];
