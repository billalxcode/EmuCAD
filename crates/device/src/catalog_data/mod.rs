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
    entries.extend_from_slice(passive::entries());
    entries.extend_from_slice(source::entries());
    entries.extend_from_slice(diode::entries());
    entries.extend_from_slice(transistor::entries());
    entries.extend_from_slice(protection::entries());
    entries.extend_from_slice(switch::entries());
    entries.extend_from_slice(analog::entries());
    entries.extend_from_slice(logic::entries());
    entries.extend_from_slice(memory::entries());
    entries.extend_from_slice(mcu::entries());
    entries.extend_from_slice(power::entries());
    entries.extend_from_slice(sensor::entries());
    entries.extend_from_slice(actuator::entries());
    entries.extend_from_slice(rf::entries());
    entries.extend_from_slice(behavioral::entries());
    entries.extend_from_slice(board::entries());
    entries
});

pub(crate) fn entries() -> &'static [ComponentSpec] {
    &CATALOG
}
