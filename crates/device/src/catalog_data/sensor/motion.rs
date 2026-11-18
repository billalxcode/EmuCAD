//! Generated sensor motion component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.motion.accelerometer",
        name: "accelerometer",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::Accelerometer),
        pins: 3,
        parameter: "sensitivity",
        default: 0.050986,
        unit: "V/(m/s2)",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.motion.gyroscope",
        name: "gyroscope",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::Gyroscope),
        pins: 3,
        parameter: "sensitivity",
        default: 0.01,
        unit: "V/(deg/s)",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.motion.imu",
        name: "IMU",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::Imu),
        pins: 3,
        parameter: "sensitivity",
        default: 0.050986,
        unit: "V/(m/s2)",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.motion.tilt_sensor",
        name: "tilt sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::TiltSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.025,
        unit: "V/deg",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.motion.vibration_sensor",
        name: "vibration sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::VibrationSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.050986,
        unit: "V/(m/s2)",
        variant: 0,
    },
];
