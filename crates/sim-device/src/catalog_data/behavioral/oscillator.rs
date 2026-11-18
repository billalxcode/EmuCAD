//! Generated behavioral oscillator component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "behavioral.oscillator.vco",
        name: "VCO",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralVco),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.oscillator.pll",
        name: "PLL",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralPll),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.oscillator.amplitude_modulator",
        name: "amplitude modulator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralAmplitude),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.oscillator.frequency_modulator",
        name: "frequency modulator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralFrequency),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.oscillator.pulse_width_modulator",
        name: "pulse width modulator",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralPwm),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
];
