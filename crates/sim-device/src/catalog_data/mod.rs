//! Generated component catalog aggregator.

mod actuator;
mod analog;
mod behavioral;
mod board;
mod diode;
mod logic;
mod mcu;
mod memory;
mod passive;
mod power;
mod protection;
mod rf;
mod sensor;
mod source;
mod switch;
mod transistor;

use crate::catalog::ComponentSpec;
use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<ComponentSpec>> = LazyLock::new(|| {
    let mut entries = Vec::new();
    entries.extend_from_slice(passive::ENTRIES);
    entries.extend_from_slice(source::ENTRIES);
    entries.extend_from_slice(diode::ENTRIES);
    entries.extend_from_slice(transistor::ENTRIES);
    entries.extend_from_slice(protection::ENTRIES);
    entries.extend_from_slice(switch::ENTRIES);
    entries.extend_from_slice(analog::ENTRIES);
    entries.extend_from_slice(logic::ENTRIES);
    entries.extend_from_slice(memory::ENTRIES);
    entries.extend_from_slice(mcu::ENTRIES);
    entries.extend_from_slice(power::ENTRIES);
    entries.extend_from_slice(sensor::ENTRIES);
    entries.extend_from_slice(actuator::ENTRIES);
    entries.extend_from_slice(rf::ENTRIES);
    entries.extend_from_slice(behavioral::ENTRIES);
    entries.extend_from_slice(board::ENTRIES);
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
