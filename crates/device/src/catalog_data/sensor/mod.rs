//! Generated sensor component catalog data.

mod current;
mod humidity;
mod light;
mod magnetic;
mod motion;
mod pressure;
mod proximity;
mod temperature;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(temperature::ENTRIES);
    entries.extend_from_slice(light::ENTRIES);
    entries.extend_from_slice(magnetic::ENTRIES);
    entries.extend_from_slice(motion::ENTRIES);
    entries.extend_from_slice(pressure::ENTRIES);
    entries.extend_from_slice(humidity::ENTRIES);
    entries.extend_from_slice(current::ENTRIES);
    entries.extend_from_slice(proximity::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
