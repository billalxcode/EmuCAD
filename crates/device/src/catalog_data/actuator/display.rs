//! Generated actuator display component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "actuator.display.lcd_character_display",
        name: "LCD character display",
        class: ModelClass::Display,
        device: Some(DeviceKind::DisplayModule),
        pins: 4,
        parameter: "supply_current",
        default: 0.002,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.display.lcd_graphic_display",
        name: "LCD graphic display",
        class: ModelClass::Display,
        device: Some(DeviceKind::DisplayModule),
        pins: 4,
        parameter: "supply_current",
        default: 0.005,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.display.oled_display",
        name: "OLED display",
        class: ModelClass::Display,
        device: Some(DeviceKind::DisplayModule),
        pins: 4,
        parameter: "supply_current",
        default: 0.02,
        unit: "A",
        variant: 0,
    },
    ComponentSpec {
        id: "actuator.display.eink_display",
        name: "e-ink display",
        class: ModelClass::Display,
        device: Some(DeviceKind::DisplayModule),
        pins: 4,
        parameter: "supply_current",
        default: 0.001,
        unit: "A",
        variant: 0,
    },
];
