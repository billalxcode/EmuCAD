//! Generated actuator component catalog data.

mod buzzer;
mod display;
mod heater;
mod led;
mod motor;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(led::ENTRIES);
    entries.extend_from_slice(buzzer::ENTRIES);
    entries.extend_from_slice(motor::ENTRIES);
    entries.extend_from_slice(display::ENTRIES);
    entries.extend_from_slice(heater::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
