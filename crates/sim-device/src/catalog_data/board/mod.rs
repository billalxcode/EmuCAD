//! Generated board component catalog data.

mod arduino;
mod breadboard;
mod mechanical;
mod module;
mod power;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(arduino::ENTRIES);
    entries.extend_from_slice(breadboard::ENTRIES);
    entries.extend_from_slice(power::ENTRIES);
    entries.extend_from_slice(module::ENTRIES);
    entries.extend_from_slice(mechanical::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
