//! Generated board breadboard component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "board.breadboard.breadboard",
        name: "breadboard",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 0.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.breadboard.jumper_wire",
        name: "jumper wire",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 0.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.breadboard.jumper_header",
        name: "jumper header",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 0.0,
        unit: "V",
        variant: 0,
    },
];
