//! Generated transistor component catalog data.

mod bjt_array;
mod bjt_npn;
mod bjt_pnp;
mod jfet_n;
mod jfet_p;
mod mosfet_n;
mod mosfet_p;
mod other;
mod power;
mod power_semiconductor;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(bjt_npn::ENTRIES);
    entries.extend_from_slice(bjt_pnp::ENTRIES);
    entries.extend_from_slice(bjt_array::ENTRIES);
    entries.extend_from_slice(jfet_n::ENTRIES);
    entries.extend_from_slice(jfet_p::ENTRIES);
    entries.extend_from_slice(mosfet_n::ENTRIES);
    entries.extend_from_slice(mosfet_p::ENTRIES);
    entries.extend_from_slice(power::ENTRIES);
    entries.extend_from_slice(power_semiconductor::ENTRIES);
    entries.extend_from_slice(other::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
