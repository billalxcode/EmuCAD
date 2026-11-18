//! Generated diode component catalog data.

mod bridge;
mod led;
mod opto;
mod other;
mod photo;
mod rectifier;
mod tvs;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(rectifier::ENTRIES);
    entries.extend_from_slice(led::ENTRIES);
    entries.extend_from_slice(photo::ENTRIES);
    entries.extend_from_slice(opto::ENTRIES);
    entries.extend_from_slice(tvs::ENTRIES);
    entries.extend_from_slice(bridge::ENTRIES);
    entries.extend_from_slice(other::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
