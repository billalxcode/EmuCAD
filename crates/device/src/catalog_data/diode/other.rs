//! Generated diode other component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "diode.other.backward_diode",
        name: "backward diode",
        class: ModelClass::Diode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.other.gunn_diode",
        name: "Gunn diode",
        class: ModelClass::Diode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.other.impatt_diode",
        name: "IMPATT diode",
        class: ModelClass::Diode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.other.step_recovery_diode",
        name: "step recovery diode",
        class: ModelClass::Diode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
];
