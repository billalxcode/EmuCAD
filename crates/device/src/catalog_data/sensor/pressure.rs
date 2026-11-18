//! Generated sensor pressure component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.pressure.strain_gauge_sensor",
        name: "strain gauge sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::StrainGaugeSensor),
        pins: 3,
        parameter: "gauge_factor",
        default: 2.1,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.pressure.load_cell",
        name: "load cell",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::LoadCell),
        pins: 3,
        parameter: "rated_output",
        default: 0.002,
        unit: "V/V",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.pressure.pressure_sensor",
        name: "pressure sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::PressureSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 1e-05,
        unit: "V/Pa",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.pressure.force_sensor",
        name: "force sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::ForceSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.01,
        unit: "V/N",
        variant: 0,
    },
];
