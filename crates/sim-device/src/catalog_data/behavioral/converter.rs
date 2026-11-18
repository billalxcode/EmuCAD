//! Generated behavioral converter component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "behavioral.converter.sample_and_hold",
        name: "sample and hold",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralSampleHold),
        pins: 4,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.converter.sar_adc_behavioral",
        name: "SAR ADC behavioral",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralSarAdc),
        pins: 4,
        parameter: "resolution",
        default: 8.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.converter.delta_sigma_adc_behavioral",
        name: "delta sigma ADC behavioral",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralDeltaSigmaAdc),
        pins: 4,
        parameter: "resolution",
        default: 8.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.converter.r2r_dac_behavioral",
        name: "R-2R DAC behavioral",
        class: ModelClass::BehavioralBlock,
        device: None,
        pins: 4,
        parameter: "resolution",
        default: 8.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "behavioral.converter.pwm_dac",
        name: "PWM DAC",
        class: ModelClass::BehavioralBlock,
        device: Some(DeviceKind::BehavioralPwmDac),
        pins: 4,
        parameter: "time_constant",
        default: 0.001,
        unit: "s",
        variant: 0,
    },
];
