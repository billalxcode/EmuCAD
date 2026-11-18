//! Generated sensor proximity component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.proximity.inductive_proximity_sensor",
        name: "inductive proximity sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::InductiveProximitySensor),
        pins: 3,
        parameter: "threshold",
        default: 5.0,
        unit: "mm",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.proximity.capacitive_touch_sensor",
        name: "capacitive touch sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::CapacitiveTouchSensor),
        pins: 3,
        parameter: "threshold",
        default: 10.0,
        unit: "pF",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.proximity.ultrasonic_distance_sensor",
        name: "ultrasonic distance sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::UltrasonicDistanceSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 1.0,
        unit: "V/m",
        variant: 0,
    },
];
