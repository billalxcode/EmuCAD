//! Generated switch component catalog data.

mod analog;
mod connector;
mod mechanical;
mod relay;
mod ssr;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(mechanical::ENTRIES);
    entries.extend_from_slice(relay::ENTRIES);
    entries.extend_from_slice(ssr::ENTRIES);
    entries.extend_from_slice(analog::ENTRIES);
    entries.extend_from_slice(connector::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
