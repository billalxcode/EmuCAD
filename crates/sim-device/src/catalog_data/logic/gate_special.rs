//! Generated logic gate_special component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "logic.gate_special.open_drain_buffer",
        name: "open drain buffer",
        class: ModelClass::LogicGateSpecial,
        device: None,
        pins: 4,
        parameter: "propagation_delay",
        default: 1e-08,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "logic.gate_special.tristate_buffer",
        name: "tri-state buffer",
        class: ModelClass::LogicGateSpecial,
        device: None,
        pins: 4,
        parameter: "propagation_delay",
        default: 1e-08,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "logic.gate_special.transmission_gate",
        name: "transmission gate",
        class: ModelClass::LogicGateSpecial,
        device: None,
        pins: 4,
        parameter: "propagation_delay",
        default: 1e-08,
        unit: "s",
        variant: 0,
    },
];
