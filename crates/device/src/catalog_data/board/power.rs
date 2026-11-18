//! Generated board power component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "board.power.usb_power_module",
        name: "USB power module",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.power.dcdc_module",
        name: "DC-DC module",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "board.power.battery_holder",
        name: "battery holder",
        class: ModelClass::BoardModule,
        device: None,
        pins: 2,
        parameter: "voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
];
