//! Generated sensor current component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "sensor.current.current_sense_resistor",
        name: "current sense resistor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::CurrentSenseResistor),
        pins: 3,
        parameter: "shunt_resistance",
        default: 0.1,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.current.hall_current_sensor",
        name: "Hall current sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::HallCurrentSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.1,
        unit: "V/A",
        variant: 0,
    },
    ComponentSpec {
        id: "sensor.current.voltage_divider_sensor",
        name: "voltage divider sensor",
        class: ModelClass::Sensor,
        device: Some(DeviceKind::VoltageDividerSensor),
        pins: 3,
        parameter: "sensitivity",
        default: 0.2,
        unit: "V/V",
        variant: 0,
    },
];
