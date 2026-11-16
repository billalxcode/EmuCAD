//! Analysis selection and persisted analysis settings.

use crate::transient::TransientSettings;
use serde::{Deserialize, Serialize};

/// Simulation domain selected by the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimulationDomain {
    #[default]
    Analog,
    Avr,
    Mixed,
}

impl SimulationDomain {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Analog => "Analog",
            Self::Avr => "AVR",
            Self::Mixed => "Mixed",
        }
    }
}

/// Selected analysis for the simulation panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisKind {
    #[default]
    Dc,
    Transient,
}

impl AnalysisKind {
    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Dc => "DC operating point",
            Self::Transient => "Transient",
        }
    }
}

/// Analysis settings persisted with a project.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisState {
    pub domain: SimulationDomain,
    pub kind: AnalysisKind,
    pub transient: TransientSettings,
}
