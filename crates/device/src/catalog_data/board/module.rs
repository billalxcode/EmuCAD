//! Generated board module component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "board.module.oled_module",
        name: "OLED module",
        class: ModelClass::BoardModule,
        device: None,
        pins: 4,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.module.sensor_module",
        name: "sensor module",
        class: ModelClass::BoardModule,
        device: None,
        pins: 4,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.module.relay_module",
        name: "relay module",
        class: ModelClass::BoardModule,
        device: None,
        pins: 4,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
];
