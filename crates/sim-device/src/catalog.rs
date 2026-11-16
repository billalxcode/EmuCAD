//! Generic component catalog with model classes and validation.

#[path = "catalog_data/mod.rs"]
mod catalog_data;
mod model;
mod records;
mod registry;
#[cfg(test)]
mod tests;

pub use model::{
    CatalogError, ComponentSpec, Family, ModelClass, ParameterKind, ParameterSpec, PinRole,
    device_parameters, device_pin_roles,
};
pub use records::{CatalogRecord, catalog_records, find_catalog_record};
pub use registry::{ComponentRegistry, ComponentRequest};

/// Returns all 500 generic component entries.
#[must_use]
pub fn catalog() -> &'static [ComponentSpec] {
    catalog_data::entries()
}
