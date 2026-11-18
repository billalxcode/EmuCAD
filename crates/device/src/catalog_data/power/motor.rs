//! Generated power motor component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "power.motor.hbridge",
        name: "H-bridge",
        class: ModelClass::MotorDriver,
        device: Some(DeviceKind::MotorDriver),
        pins: 4,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.motor.halfbridge",
        name: "half-bridge",
        class: ModelClass::MotorDriver,
        device: Some(DeviceKind::MotorDriver),
        pins: 4,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.motor.stepper_driver",
        name: "stepper driver",
        class: ModelClass::MotorDriver,
        device: Some(DeviceKind::MotorDriver),
        pins: 4,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
];
