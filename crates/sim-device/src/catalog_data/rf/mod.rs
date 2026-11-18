//! Generated rf component catalog data.

mod amplifier;
mod antenna;
mod coupler;
mod filter;
mod mixer;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(amplifier::ENTRIES);
    entries.extend_from_slice(mixer::ENTRIES);
    entries.extend_from_slice(filter::ENTRIES);
    entries.extend_from_slice(antenna::ENTRIES);
    entries.extend_from_slice(coupler::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
