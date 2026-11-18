//! Generated sensor light component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.light.photoresistor",
        name: "photoresistor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::Photoresistor),
        pins: 3,
        parameter: "resistance_10lux",
        default: 10000.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.light.photodiode_sensor",
        name: "photodiode sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::PhotodiodeSensor),
        pins: 3,
        parameter: "responsivity",
        default: 0.5,
        unit: "A/W",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.light.phototransistor_sensor",
        name: "phototransistor sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::LinearLightSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.01,
        unit: "V/lux",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.light.ambient_light_sensor",
        name: "ambient light sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::LinearLightSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.001,
        unit: "V/lux",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.light.color_sensor",
        name: "color sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::LinearLightSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.001,
        unit: "V/lux",
        variant: 0,
    },
];
