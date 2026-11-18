//! Generated diode bridge component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "diode.bridge.diode_bridge",
        name: "diode bridge",
        class: ModelClass::DiodeArray,
        device: Some(DeviceKind::DiodeBridge),
        pins: 4,
        parameter: "forward_voltage",
        default: 0.7,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.bridge.dual_diode",
        name: "dual diode",
        class: ModelClass::DiodeArray,
        device: Some(DeviceKind::DualDiode),
        pins: 4,
        parameter: "forward_voltage",
        default: 0.7,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.bridge.diode_array",
        name: "diode array",
        class: ModelClass::DiodeArray,
        device: Some(DeviceKind::DiodeBridge),
        pins: 4,
        parameter: "forward_voltage",
        default: 0.7,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.bridge.schottky_bridge",
        name: "Schottky bridge",
        class: ModelClass::DiodeArray,
        device: Some(DeviceKind::DiodeBridge),
        pins: 4,
        parameter: "forward_voltage",
        default: 0.3,
        unit: "V",
        variant: 0,
    },
];
