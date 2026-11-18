//! Generated behavioral detector component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "behavioral.detector.behavioral_comparator",
        name: "behavioral comparator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralComparator),
        pins: 4,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.detector.schmitt_trigger",
        name: "Schmitt trigger",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralSchmitt),
        pins: 4,
        parameter: "upper_threshold",
        default: 3.0,
        unit: "V",
        variant: 8,
    },
    ComponentSpec {
        id: "behavioral.detector.peak_detector",
        name: "peak detector",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralPeakDetector),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.detector.track_and_hold",
        name: "track and hold",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralTrackHold),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.detector.precision_rectifier",
        name: "precision rectifier",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralPrecisionRectifier),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
