//! Generated actuator led component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "actuator.led.led",
        name: "LED",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::LedActuator),
        pins: 2,
        parameter: "is",
        default: 1e-15,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.led.rgb_led",
        name: "RGB LED",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::LedActuator),
        pins: 2,
        parameter: "is",
        default: 1e-15,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.led.incandescent_lamp",
        name: "incandescent lamp",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 100.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.led.neon_lamp",
        name: "neon lamp",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Diac),
        pins: 2,
        parameter: "vbo",
        default: 90.0,
        unit: "V",
        variant: 0,
    },
];
