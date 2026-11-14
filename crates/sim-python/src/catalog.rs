//! Catalog records exposed to Python.

use sim_device::{ComponentRegistry, ComponentSpec, catalog};

/// One catalog entry in a Python-friendly form.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CatalogEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub model_class: String,
    pub device: Option<String>,
    pub pins: u8,
    pub parameter: &'static str,
    pub default: f64,
    pub unit: &'static str,
    pub variant: u8,
    pub solver_backed: bool,
}

impl CatalogEntry {
    fn from_spec(spec: &ComponentSpec) -> Self {
        Self {
            id: spec.id,
            name: spec.name,
            family: spec.class.family().name(),
            model_class: format!("{:?}", spec.class),
            device: spec.device.as_ref().map(|kind| kind.name().to_string()),
            pins: spec.pins,
            parameter: spec.parameter,
            default: spec.default,
            unit: spec.unit,
            variant: spec.variant,
            solver_backed: spec.device.is_some(),
        }
    }
}

/// Returns all 500 catalog entries.
pub(crate) fn entries() -> Vec<CatalogEntry> {
    catalog().iter().map(CatalogEntry::from_spec).collect()
}

/// Finds one entry by ID.
pub(crate) fn find(id: &str) -> Option<CatalogEntry> {
    ComponentRegistry::new()
        .find(id)
        .map(CatalogEntry::from_spec)
}
