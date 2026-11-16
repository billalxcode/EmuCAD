//! Analysis selection and persisted analysis settings.

use crate::transient::TransientSettings;
use serde::{Deserialize, Serialize};

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
    pub kind: AnalysisKind,
    pub transient: TransientSettings,
}
