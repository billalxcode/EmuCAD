//! Generated logic component catalog data.

mod adder;
mod counter;
mod flip_flop;
mod level_shifter;
mod logic_gate;
mod logic_gate_special;
mod multiplexer;
mod programmable_logic;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(logic_gate::ENTRIES);
    entries.extend_from_slice(logic_gate_special::ENTRIES);
    entries.extend_from_slice(flip_flop::ENTRIES);
    entries.extend_from_slice(counter::ENTRIES);
    entries.extend_from_slice(multiplexer::ENTRIES);
    entries.extend_from_slice(adder::ENTRIES);
    entries.extend_from_slice(level_shifter::ENTRIES);
    entries.extend_from_slice(programmable_logic::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
