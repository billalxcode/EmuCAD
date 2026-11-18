//! Generated analog ota component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "analog.ota.ota",
        name: "OTA",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1000.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.ota.norton_amplifier",
        name: "Norton amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1000.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.ota.audio_power_amplifier",
        name: "audio power amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1000.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.ota.video_amplifier",
        name: "video amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1000.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.ota.buffer_amplifier",
        name: "buffer amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1000.0,
        unit: "ratio",
        variant: 0,
    },
];
