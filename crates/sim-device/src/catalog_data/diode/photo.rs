//! Generated diode photo component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "diode.photo.photodiode",
        name: "photodiode",
        class: ModelClass::Photodiode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "diode.photo.avalanche_photodiode",
        name: "avalanche photodiode",
        class: ModelClass::Photodiode,
        device: Some(DeviceKind::Diode),
        pins: 2,
        parameter: "is",
        default: 1e-12,
        unit: "A",
        variant: 0,
    },
];
