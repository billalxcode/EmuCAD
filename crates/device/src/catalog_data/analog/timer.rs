//! Generated analog timer component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "analog.timer.555_timer",
        name: "555 timer",
        class: ModelClass::Timer,
        device: Some(DeviceKind::Timer555),
        pins: 8,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.timer.monostable_multivibrator",
        name: "monostable multivibrator",
        class: ModelClass::Timer,
        device: Some(DeviceKind::MonostableTimer),
        pins: 8,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.timer.astable_multivibrator",
        name: "astable multivibrator",
        class: ModelClass::Timer,
        device: Some(DeviceKind::AstableTimer),
        pins: 8,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.timer.voltage_controlled_oscillator",
        name: "voltage controlled oscillator",
        class: ModelClass::Timer,
        device: Some(DeviceKind::VcoTimer),
        pins: 8,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "analog.timer.pll",
        name: "PLL",
        class: ModelClass::Timer,
        device: Some(DeviceKind::TimerPll),
        pins: 8,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
];
