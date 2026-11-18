//! Generated mcu component catalog data.

mod converter;
mod core32;
mod core8;
mod serial;
mod timer;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(core8::ENTRIES);
    entries.extend_from_slice(core32::ENTRIES);
    entries.extend_from_slice(converter::ENTRIES);
    entries.extend_from_slice(serial::ENTRIES);
    entries.extend_from_slice(timer::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
