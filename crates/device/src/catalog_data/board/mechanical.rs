//! Generated board mechanical component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "board.mechanical.enclosure",
        name: "enclosure",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 0.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.mechanical.mounting_hardware",
        name: "mounting hardware",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 0.0,
        unit: "V",
        variant: 0,
    },
];
