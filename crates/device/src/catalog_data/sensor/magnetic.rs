//! Generated sensor magnetic component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.magnetic.hall_sensor",
        name: "Hall sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::HallSwitch),
        pins: 3,
        parameter: "threshold",
        default: 0.005,
        unit: "T",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.magnetic.linear_hall_sensor",
        name: "linear Hall sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::LinearHallSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 5.0,
        unit: "V/T",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.magnetic.reed_switch_sensor",
        name: "reed switch sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::ReedSwitchSensor),
        pins: 3,
        parameter: "threshold",
        default: 0.005,
        unit: "T",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.magnetic.magnetoresistive_sensor",
        name: "magnetoresistive sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::MagnetoresistiveSensor),
        pins: 3,
        parameter: "resistance",
        default: 1000.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.magnetic.fluxgate_sensor",
        name: "fluxgate sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::FluxgateSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 1000.0,
        unit: "V/T",
        variant: 0,
    },
];
