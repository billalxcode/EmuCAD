//! Generated source controlled component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "source.controlled.vcvs",
        name: "VCVS",
        class: ModelClass::Vcvs,
        device: Some(DeviceKind::Vcvs),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "source.controlled.vccs",
        name: "VCCS",
        class: ModelClass::Vccs,
        device: Some(DeviceKind::Vccs),
        pins: 4,
        parameter: "gm",
        default: 0.001,
        unit: "S",
        variant: 0,
    },
    ComponentSpec {
        id: "source.controlled.cccs",
        name: "CCCS",
        class: ModelClass::Cccs,
        device: Some(DeviceKind::Cccs),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "source.controlled.ccvs",
        name: "CCVS",
        class: ModelClass::Ccvs,
        device: Some(DeviceKind::Ccvs),
        pins: 4,
        parameter: "rm",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
];
