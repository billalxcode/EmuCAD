//! Generated behavioral filter component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "behavioral.filter.integrator",
        name: "integrator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralIntegrator),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.filter.differentiator",
        name: "differentiator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralDifferentiator),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.filter.first_order_low_pass",
        name: "first order low pass",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralLowPass),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.filter.first_order_high_pass",
        name: "first order high pass",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralHighPass),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.filter.state_variable_filter",
        name: "state variable filter",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralStateVariable),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
