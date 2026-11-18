//! Generated power switch component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "power.switch.ideal_diode",
        name: "ideal diode",
        class: ModelClass::PowerSwitch,
        device: Some(DeviceKind::IdealSwitch),
        pins: 2,
        parameter: "ron",
        default: 0.05,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "power.switch.oring_controller",
        name: "ORing controller",
        class: ModelClass::PowerSwitch,
        device: Some(DeviceKind::IdealSwitch),
        pins: 2,
        parameter: "ron",
        default: 0.05,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "power.switch.load_switch",
        name: "load switch",
        class: ModelClass::PowerSwitch,
        device: Some(DeviceKind::IdealSwitch),
        pins: 2,
        parameter: "ron",
        default: 0.05,
        unit: "ohm",
        variant: 0,
    },
];
