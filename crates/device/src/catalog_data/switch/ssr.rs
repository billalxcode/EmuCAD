//! Generated switch ssr component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use kernel::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "switch.ssr.solid_state_relay",
        name: "solid state relay",
        class: ModelClass::SolidStateRelay,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "vth",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.ssr.opto_mosfet_relay",
        name: "opto MOSFET relay",
        class: ModelClass::SolidStateRelay,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "vth",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.ssr.triac_output_ssr",
        name: "TRIAC output SSR",
        class: ModelClass::SolidStateRelay,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "vth",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
    ComponentSpec {
        id: "switch.ssr.time_delay_relay",
        name: "time delay relay",
        class: ModelClass::SolidStateRelay,
        device: Some(DeviceKind::Switch),
        pins: 4,
        parameter: "vth",
        default: 2.5,
        unit: "V",
        variant: 0,
    },
];
