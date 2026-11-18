//! Generated sensor humidity component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.humidity.humidity_sensor",
        name: "humidity sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::HumiditySensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.03,
        unit: "V/%RH",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.humidity.gas_sensor",
        name: "gas sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::GasSensor),
        pins: 3,
        parameter: "resistance",
        default: 10000.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.humidity.ph_sensor",
        name: "pH sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::PhSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.05916,
        unit: "V/pH",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.humidity.electrochemical_sensor",
        name: "electrochemical sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::ElectrochemicalSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.01,
        unit: "V/ppm",
        variant: 0,
    },
];
