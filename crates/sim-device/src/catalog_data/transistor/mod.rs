//! Generated transistor component catalog data.

mod bjt;
mod jfet;
mod mosfet;
mod other_transistor;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(bjt::ENTRIES);
    entries.extend_from_slice(jfet::ENTRIES);
    entries.extend_from_slice(mosfet::ENTRIES);
    entries.extend_from_slice(other_transistor::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
