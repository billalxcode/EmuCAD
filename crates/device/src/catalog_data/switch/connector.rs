//! Generated switch connector component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "switch.connector.header_connector",
        name: "header connector",
        class: ModelClass::Connector,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 0.001,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.connector.terminal_block",
        name: "terminal block",
        class: ModelClass::Connector,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 0.001,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.connector.jumper",
        name: "jumper",
        class: ModelClass::Connector,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 0.001,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.connector.test_point",
        name: "test point",
        class: ModelClass::Connector,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 0.001,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.connector.ic_socket",
        name: "IC socket",
        class: ModelClass::Connector,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 0.001,
        unit: "ohm",
        variant: 0,
    },
];
