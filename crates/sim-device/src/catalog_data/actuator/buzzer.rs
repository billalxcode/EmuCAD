//! Generated actuator buzzer component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "actuator.buzzer.buzzer",
        name: "buzzer",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 100.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.buzzer.speaker",
        name: "speaker",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 8.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.buzzer.piezo_transducer",
        name: "piezo transducer",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Capacitor),
        pins: 2,
        parameter: "capacitance",
        default: 1e-07,
        unit: "F",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.buzzer.piezo_buzzer",
        name: "piezo buzzer",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Capacitor),
        pins: 2,
        parameter: "capacitance",
        default: 2e-08,
        unit: "F",
        variant: 0,
    },
];
