//! Generated actuator motor component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "actuator.motor.dc_motor",
        name: "DC motor",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::DcMotor),
        pins: 2,
        parameter: "resistance",
        default: 10.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.motor.brushless_dc_motor",
        name: "brushless DC motor",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::DcMotor),
        pins: 2,
        parameter: "resistance",
        default: 5.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.motor.servo_motor",
        name: "servo motor",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::DcMotor),
        pins: 2,
        parameter: "resistance",
        default: 50.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.motor.stepper_motor",
        name: "stepper motor",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::DcMotor),
        pins: 2,
        parameter: "resistance",
        default: 20.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.motor.solenoid",
        name: "solenoid",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 20.0,
        unit: "ohm",
        variant: 0,
    },
];
