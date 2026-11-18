//! Generated analog component catalog data.

mod comparator;
mod instrumentation;
mod opamp;
mod ota;
mod reference;
mod timer;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(opamp::ENTRIES);
    entries.extend_from_slice(comparator::ENTRIES);
    entries.extend_from_slice(instrumentation::ENTRIES);
    entries.extend_from_slice(ota::ENTRIES);
    entries.extend_from_slice(timer::ENTRIES);
    entries.extend_from_slice(reference::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
