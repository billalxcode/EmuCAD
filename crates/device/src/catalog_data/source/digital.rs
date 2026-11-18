//! Generated source digital component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "source.digital.digital_clock_source",
        name: "digital clock source",
        class: ModelClass::DigitalSource,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "source.digital.digital_pattern_source",
        name: "digital pattern source",
        class: ModelClass::DigitalSource,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "source.digital.prbs_source",
        name: "PRBS source",
        class: ModelClass::DigitalSource,
        device: None,
        pins: 1,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
];
