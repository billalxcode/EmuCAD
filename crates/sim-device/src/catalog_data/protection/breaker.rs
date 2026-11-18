//! Generated protection breaker component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "protection.breaker.spark_gap",
        name: "spark gap",
        class: ModelClass::Protection,
        device: Some(DeviceKind::Varistor),
        pins: 2,
        parameter: "clamp_voltage",
        default: 100.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "protection.breaker.crowbar",
        name: "crowbar",
        class: ModelClass::Protection,
        device: Some(DeviceKind::Varistor),
        pins: 2,
        parameter: "clamp_voltage",
        default: 100.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "protection.breaker.surge_arrester",
        name: "surge arrester",
        class: ModelClass::Protection,
        device: Some(DeviceKind::Varistor),
        pins: 2,
        parameter: "clamp_voltage",
        default: 100.0,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "protection.breaker.circuit_breaker",
        name: "circuit breaker",
        class: ModelClass::Protection,
        device: Some(DeviceKind::Fuse),
        pins: 2,
        parameter: "trip_current",
        default: 10.0,
        unit: "A",
        variant: 0,
    },
];
