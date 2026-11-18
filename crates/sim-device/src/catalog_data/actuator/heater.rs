//! Generated actuator heater component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "actuator.heater.resistive_heater",
        name: "resistive heater",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Resistor),
        pins: 2,
        parameter: "resistance",
        default: 10.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.heater.peltier_cooler",
        name: "Peltier cooler",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::Peltier),
        pins: 2,
        parameter: "resistance",
        default: 2.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.heater.thermoelectric_generator",
        name: "thermoelectric generator",
        class: ModelClass::Actuator,
        device: Some(DeviceKind::ThermoelectricGenerator),
        pins: 2,
        parameter: "seebeck",
        default: 0.05,
        unit: "V/K",
        variant: 0,
    },
];
