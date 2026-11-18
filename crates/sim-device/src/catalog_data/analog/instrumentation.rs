//! Generated analog instrumentation component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "analog.instrumentation.instrumentation_amplifier",
        name: "instrumentation amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.instrumentation.isolation_amplifier",
        name: "isolation amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.instrumentation.current_sense_amplifier",
        name: "current sense amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.instrumentation.difference_amplifier",
        name: "difference amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.instrumentation.programmable_gain_amplifier",
        name: "programmable gain amplifier",
        class: ModelClass::OpAmp,
        device: Some(DeviceKind::OpAmp),
        pins: 5,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
