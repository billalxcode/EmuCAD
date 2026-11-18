//! Generated source component catalog data.

mod battery;
mod behavioral;
mod controlled;
mod digital;
mod independent;
mod pulse;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(independent::ENTRIES);
    entries.extend_from_slice(pulse::ENTRIES);
    entries.extend_from_slice(controlled::ENTRIES);
    entries.extend_from_slice(behavioral::ENTRIES);
    entries.extend_from_slice(digital::ENTRIES);
    entries.extend_from_slice(battery::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
