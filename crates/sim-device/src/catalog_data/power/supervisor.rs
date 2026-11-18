//! Generated power supervisor component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "power.supervisor.voltage_reference",
        name: "voltage reference",
        class: ModelClass::Supervisor,
        device: Some(DeviceKind::VoltageReference),
        pins: 3,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.supervisor.current_reference",
        name: "current reference",
        class: ModelClass::Supervisor,
        device: Some(DeviceKind::CurrentReference),
        pins: 3,
        parameter: "output_current",
        default: 0.001,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "power.supervisor.voltage_supervisor",
        name: "voltage supervisor",
        class: ModelClass::Supervisor,
        device: Some(DeviceKind::Supervisor),
        pins: 3,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "power.supervisor.reset_supervisor",
        name: "reset supervisor",
        class: ModelClass::Supervisor,
        device: Some(DeviceKind::Supervisor),
        pins: 3,
        parameter: "active_low",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "power.supervisor.power_monitor",
        name: "power monitor",
        class: ModelClass::Supervisor,
        device: Some(DeviceKind::Supervisor),
        pins: 3,
        parameter: "threshold",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
];
