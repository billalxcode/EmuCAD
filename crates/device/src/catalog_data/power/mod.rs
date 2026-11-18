//! Generated power component catalog data.

mod charge;
mod dcdc;
mod linear;
mod motor;
mod supervisor;
mod switch;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(linear::ENTRIES);
    entries.extend_from_slice(dcdc::ENTRIES);
    entries.extend_from_slice(charge::ENTRIES);
    entries.extend_from_slice(supervisor::ENTRIES);
    entries.extend_from_slice(switch::ENTRIES);
    entries.extend_from_slice(motor::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
