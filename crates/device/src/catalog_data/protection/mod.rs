//! Generated protection component catalog data.

mod breaker;
mod device;
mod thyristor;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(thyristor::ENTRIES);
    entries.extend_from_slice(device::ENTRIES);
    entries.extend_from_slice(breaker::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
