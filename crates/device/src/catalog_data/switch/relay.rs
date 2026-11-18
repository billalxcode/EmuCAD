//! Generated switch relay component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "switch.relay.electromechanical_relay",
        name: "electromechanical relay",
        class: ModelClass::Relay,
        device: Some(DeviceKind::Relay),
        pins: 5,
        parameter: "rcoil",
        default: 100.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.relay.latching_relay",
        name: "latching relay",
        class: ModelClass::Relay,
        device: Some(DeviceKind::Relay),
        pins: 5,
        parameter: "rcoil",
        default: 100.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.relay.reed_relay",
        name: "reed relay",
        class: ModelClass::Relay,
        device: Some(DeviceKind::Relay),
        pins: 5,
        parameter: "rcoil",
        default: 100.0,
        unit: "ohm",
        variant: 0,
    },
];
