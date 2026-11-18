//! Generated passive component catalog data.

mod capacitor;
mod crystal;
mod inductor;
mod resistor;
mod transformer;
mod transmission_line;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(resistor::ENTRIES);
    entries.extend_from_slice(capacitor::ENTRIES);
    entries.extend_from_slice(inductor::ENTRIES);
    entries.extend_from_slice(transformer::ENTRIES);
    entries.extend_from_slice(crystal::ENTRIES);
    entries.extend_from_slice(transmission_line::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
