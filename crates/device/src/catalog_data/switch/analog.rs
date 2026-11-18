//! Generated switch analog component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "switch.analog.analog_switch",
        name: "analog switch",
        class: ModelClass::AnalogSwitch,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "ron",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.analog.spdt_analog_switch",
        name: "SPDT analog switch",
        class: ModelClass::AnalogSwitch,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "ron",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.analog.4_to_1_analog_multiplexer",
        name: "4 to 1 analog multiplexer",
        class: ModelClass::AnalogSwitch,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "ron",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.analog.8_to_1_analog_multiplexer",
        name: "8 to 1 analog multiplexer",
        class: ModelClass::AnalogSwitch,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "ron",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.analog.analog_demultiplexer",
        name: "analog demultiplexer",
        class: ModelClass::AnalogSwitch,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "ron",
        default: 1.0,
        unit: "ohm",
        variant: 0,
    },
];
