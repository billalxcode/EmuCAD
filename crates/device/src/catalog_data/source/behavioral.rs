//! Generated source behavioral component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "source.behavioral.behavioral_voltage_source",
        name: "behavioral voltage source",
        class: ModelClass::BehavioralSource,
        device: None,
        pins: 2,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "source.behavioral.behavioral_current_source",
        name: "behavioral current source",
        class: ModelClass::BehavioralSource,
        device: None,
        pins: 2,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "source.behavioral.polynomial_source",
        name: "polynomial source",
        class: ModelClass::BehavioralSource,
        device: Some(DeviceKind::PolynomialVoltageSource),
        pins: 2,
        parameter: "c1",
        default: 1.0,
        unit: "V/s",
        variant: 0,
    },
    ComponentSpec {
        id: "source.behavioral.table_source",
        name: "table source",
        class: ModelClass::BehavioralSource,
        device: Some(DeviceKind::PwlVoltageSource),
        pins: 2,
        parameter: "v1",
        default: 5.0,
        unit: "V",
        variant: 0,
    },
];
