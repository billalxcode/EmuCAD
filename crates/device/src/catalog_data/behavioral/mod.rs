//! Generated behavioral component catalog data.

mod converter;
mod detector;
mod filter;
mod math;
mod oscillator;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(filter::ENTRIES);
    entries.extend_from_slice(math::ENTRIES);
    entries.extend_from_slice(detector::ENTRIES);
    entries.extend_from_slice(oscillator::ENTRIES);
    entries.extend_from_slice(converter::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
