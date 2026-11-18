//! Generated power charge component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "power.charge.charge_pump",
        name: "charge pump",
        class: ModelClass::ChargePump,
        device: Some(DeviceKind::ChargePump),
        pins: 3,
        parameter: "output_voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.charge.inverting_charge_pump",
        name: "inverting charge pump",
        class: ModelClass::ChargePump,
        device: Some(DeviceKind::ChargePump),
        pins: 3,
        parameter: "output_voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.charge.linear_charger",
        name: "linear charger",
        class: ModelClass::ChargePump,
        device: Some(DeviceKind::ChargePump),
        pins: 3,
        parameter: "output_voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.charge.switching_charger",
        name: "switching charger",
        class: ModelClass::ChargePump,
        device: Some(DeviceKind::ChargePump),
        pins: 3,
        parameter: "output_voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.charge.solar_charger",
        name: "solar charger",
        class: ModelClass::ChargePump,
        device: Some(DeviceKind::ChargePump),
        pins: 3,
        parameter: "output_voltage",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
];
