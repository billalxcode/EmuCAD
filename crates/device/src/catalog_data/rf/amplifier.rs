//! Generated rf amplifier component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "rf.amplifier.rf_amplifier",
        name: "RF amplifier",
        class: ModelClass::RfBlock,
        device: Some(DeviceKind::RfAmplifier),
        pins: 3,
        parameter: "gain",
        default: 10.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.amplifier.low_noise_amplifier",
        name: "low noise amplifier",
        class: ModelClass::RfBlock,
        device: Some(DeviceKind::RfAmplifier),
        pins: 3,
        parameter: "gain",
        default: 100.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.amplifier.power_amplifier",
        name: "power amplifier",
        class: ModelClass::RfBlock,
        device: Some(DeviceKind::RfAmplifier),
        pins: 3,
        parameter: "gain",
        default: 2.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.amplifier.variable_gain_amplifier",
        name: "variable gain amplifier",
        class: ModelClass::RfBlock,
        device: Some(DeviceKind::RfAmplifier),
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
