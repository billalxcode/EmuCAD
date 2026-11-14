//! Generic component catalog with model classes and validation.

#[path = "catalog_data/mod.rs"]
mod catalog_data;
mod model;
mod registry;
#[cfg(test)]
mod tests;

pub use model::{CatalogError, ComponentSpec, Family, ModelClass};
pub use registry::{ComponentRegistry, ComponentRequest};

/// Returns all 500 generic component entries.
#[must_use]
pub fn catalog() -> &'static [ComponentSpec] {
    catalog_data::entries()
}
