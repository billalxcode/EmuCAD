//! Generated rf antenna component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "rf.antenna.dipole_antenna",
        name: "dipole antenna",
        class: ModelClass::RfBlock,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.antenna.monopole_antenna",
        name: "monopole antenna",
        class: ModelClass::RfBlock,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.antenna.loop_antenna",
        name: "loop antenna",
        class: ModelClass::RfBlock,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
];
