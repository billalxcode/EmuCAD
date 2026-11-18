//! Generated behavioral math component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "behavioral.math.multiplier",
        name: "multiplier",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralMultiplier),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.math.divider",
        name: "divider",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralDivider),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.math.logarithmic_amplifier",
        name: "logarithmic amplifier",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralLog),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.math.antilog_amplifier",
        name: "antilog amplifier",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralAntilog),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.math.rms_converter",
        name: "RMS converter",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralRms),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
