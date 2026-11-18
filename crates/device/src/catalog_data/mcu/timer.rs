//! Generated mcu timer component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "mcu.timer.timer",
        name: "timer",
        class: ModelClass::TimerPeripheral,
        device: None,
        pins: 4,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.timer.rtc",
        name: "RTC",
        class: ModelClass::TimerPeripheral,
        device: None,
        pins: 4,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.timer.watchdog",
        name: "watchdog",
        class: ModelClass::TimerPeripheral,
        device: None,
        pins: 4,
        parameter: "period",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
];
