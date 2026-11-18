//! Generated transistor power component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "transistor.power.power_n_mosfet",
        name: "power N MOSFET",
        class: ModelClass::Mosfet,
        device: Some(DeviceKind::Nmos),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.power.power_p_mosfet",
        name: "power P MOSFET",
        class: ModelClass::Mosfet,
        device: Some(DeviceKind::Pmos),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
];
