//! Generated rf coupler component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "rf.coupler.directional_coupler",
        name: "directional coupler",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.coupler.power_splitter",
        name: "power splitter",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.coupler.isolator",
        name: "isolator",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
