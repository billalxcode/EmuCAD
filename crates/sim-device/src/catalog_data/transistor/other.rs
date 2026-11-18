//! Generated transistor other component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "transistor.other.mesfet",
        name: "MESFET",
        class: ModelClass::OtherTransistor,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.other.hemt",
        name: "HEMT",
        class: ModelClass::OtherTransistor,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.other.ujt",
        name: "UJT",
        class: ModelClass::OtherTransistor,
        device: Some(DeviceKind::Ujt),
        pins: 3,
        parameter: "eta",
        default: 0.6,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.other.put",
        name: "PUT",
        class: ModelClass::OtherTransistor,
        device: Some(DeviceKind::Put),
        pins: 3,
        parameter: "vd",
        default: 0.7,
        unit: "V",
        variant: 0,
    },
];
