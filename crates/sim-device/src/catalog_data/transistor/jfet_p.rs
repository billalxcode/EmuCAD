//! Generated transistor jfet_p component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "transistor.jfet_p.p_jfet",
        name: "P JFET",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetP),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_p.p_jfet_switch",
        name: "P JFET switch",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetP),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_p.dual_p_jfet",
        name: "dual P JFET",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetP),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "transistor.jfet_p.p_jfet_rf",
        name: "P JFET RF",
        class: ModelClass::Jfet,
        device: Some(DeviceKind::JfetP),
        pins: 3,
        parameter: "vto",
        default: 2.0,
        unit: "V",
        variant: 0,
    },
];
