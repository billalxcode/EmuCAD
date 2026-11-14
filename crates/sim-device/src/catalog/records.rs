//! Stable catalog records shared by adapters.

use super::catalog;
use super::model::ComponentSpec;

/// One catalog entry with owned display fields.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogRecord {
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

impl CatalogRecord {
    /// Builds a record from one catalog entry.
    #[must_use]
    pub fn from_spec(spec: &ComponentSpec) -> Self {
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

/// Returns all catalog entries as records.
#[must_use]
pub fn catalog_records() -> Vec<CatalogRecord> {
    catalog().iter().map(CatalogRecord::from_spec).collect()
}

/// Finds one catalog record by ID.
#[must_use]
pub fn find_catalog_record(id: &str) -> Option<CatalogRecord> {
    catalog()
        .iter()
        .find(|spec| spec.id == id)
        .map(CatalogRecord::from_spec)
}

#[cfg(test)]
mod tests {
    use super::{CatalogRecord, catalog_records, find_catalog_record};

    #[test]
    fn records_cover_the_catalog() {
        let records = catalog_records();
        assert_eq!(records.len(), 500);
        assert_eq!(
            records.iter().filter(|record| record.solver_backed).count(),
            215
        );
    }

    #[test]
    fn resistor_record_exposes_device() {
        let record = find_catalog_record("passive.resistor.ideal_resistor").expect("resistor");
        assert_eq!(record.family, "passive");
        assert_eq!(record.device.as_deref(), Some("resistor"));
        assert!(record.solver_backed);
    }

    #[test]
    fn behavioral_record_has_no_device() {
        let record = catalog_records()
            .into_iter()
            .find(|record| !record.solver_backed)
            .expect("behavioral");
        assert_eq!(record.device, None);
    }

    #[test]
    fn missing_record_returns_none() {
        assert_eq!(find_catalog_record("missing.part"), None);
    }

    #[test]
    fn record_keeps_catalog_fields() {
        let spec = &super::catalog()[0];
        let record = CatalogRecord::from_spec(spec);
        assert_eq!(record.id, spec.id);
        assert_eq!(record.name, spec.name);
        assert_eq!(record.pins, spec.pins);
        assert_eq!(record.default.to_bits(), spec.default.to_bits());
    }
}
