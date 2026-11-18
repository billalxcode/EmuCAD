//! Generated logic component catalog data.

mod adder;
mod counter;
mod flipflop;
mod gate;
mod gate_special;
mod level;
mod mux;
mod programmable;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(gate::ENTRIES);
    entries.extend_from_slice(gate_special::ENTRIES);
    entries.extend_from_slice(flipflop::ENTRIES);
    entries.extend_from_slice(counter::ENTRIES);
    entries.extend_from_slice(mux::ENTRIES);
    entries.extend_from_slice(adder::ENTRIES);
    entries.extend_from_slice(level::ENTRIES);
    entries.extend_from_slice(programmable::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
