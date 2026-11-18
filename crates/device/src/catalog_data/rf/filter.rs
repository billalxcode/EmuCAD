//! Generated rf filter component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "rf.filter.low_pass_filter",
        name: "low pass filter",
        class: ModelClass::RfBlock,
        device: None,
        pins: 2,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.filter.high_pass_filter",
        name: "high pass filter",
        class: ModelClass::RfBlock,
        device: None,
        pins: 2,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.filter.band_pass_filter",
        name: "band pass filter",
        class: ModelClass::RfBlock,
        device: None,
        pins: 2,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.filter.band_stop_filter",
        name: "band stop filter",
        class: ModelClass::RfBlock,
        device: None,
        pins: 2,
        parameter: "frequency",
        default: 1000000.0,
        unit: "Hz",
        variant: 0,
    },
];
