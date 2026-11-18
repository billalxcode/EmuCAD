//! Generated transistor jfet_n component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "transistor.jfet_n.n_jfet",
        name: "N JFET",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_n.n_jfet_switch",
        name: "N JFET switch",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_n.dual_n_jfet",
        name: "dual N JFET",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_n.n_jfet_rf",
        name: "N JFET RF",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetN),
        pins: 3,
        parameter: "vto",
        default: -2.0,
        unit: "V",
        variant: 0,
    },
];
