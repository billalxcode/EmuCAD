//! Generated source battery component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "source.battery.battery",
        name: "battery",
        class: ModelClass::Battery,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 1.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "source.battery.solar_cell",
        name: "solar cell",
        class: ModelClass::Battery,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 1.5,
        unit: "V",
        variant: 0,
    },
];
