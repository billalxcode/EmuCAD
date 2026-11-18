//! Generated mcu converter component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};
use sim_core::DeviceKind;

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "mcu.converter.sar_adc",
        name: "SAR ADC",
        class: ModelClass::Adc,
        device: Some(DeviceKind::BehavioralSarAdc),
        pins: 4,
        parameter: "resolution",
        default: 10.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.converter.deltasigma_adc",
        name: "delta-sigma ADC",
        class: ModelClass::Adc,
        device: Some(DeviceKind::BehavioralDeltaSigmaAdc),
        pins: 4,
        parameter: "resolution",
        default: 10.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.converter.flash_adc",
        name: "flash ADC",
        class: ModelClass::Adc,
        device: Some(DeviceKind::BehavioralSarAdc),
        pins: 4,
        parameter: "resolution",
        default: 10.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.converter.r2r_dac",
        name: "R-2R DAC",
        class: ModelClass::Adc,
        device: None,
        pins: 4,
        parameter: "resolution",
        default: 10.0,
        unit: "bit",
        variant: 0,
    },
    ComponentSpec {
        id: "mcu.converter.pwm_generator",
        name: "PWM generator",
        class: ModelClass::Adc,
        device: Some(DeviceKind::BehavioralPwm),
        pins: 4,
        parameter: "frequency",
        default: 1000.0,
        unit: "Hz",
        variant: 0,
    },
];
