//! Generated analog reference component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "analog.reference.shunt_voltage_reference",
        name: "shunt voltage reference",
        class: ModelClass::VoltageReference,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.reference.series_voltage_reference",
        name: "series voltage reference",
        class: ModelClass::VoltageReference,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.reference.bandgap_reference",
        name: "bandgap reference",
        class: ModelClass::VoltageReference,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.reference.current_reference",
        name: "current reference",
        class: ModelClass::VoltageReference,
        device: Some(DeviceKind::VoltageSource),
        pins: 2,
        parameter: "voltage",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
];
