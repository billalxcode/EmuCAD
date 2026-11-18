//! Generated rf mixer component catalog data.

use crate::catalog::{ComponentSpec, ModelClass};

pub(crate) const ENTRIES: &[ComponentSpec] = &[
    ComponentSpec {
        id: "rf.mixer.rf_mixer",
        name: "RF mixer",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.mixer.amplitude_modulator",
        name: "amplitude modulator",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.mixer.frequency_modulator",
        name: "frequency modulator",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
    ComponentSpec {
        id: "rf.mixer.phase_modulator",
        name: "phase modulator",
        class: ModelClass::RfBlock,
        device: None,
        pins: 3,
        parameter: "gain",
        default: 1.0,
        unit: "ratio",
        variant: 0,
    },
];
