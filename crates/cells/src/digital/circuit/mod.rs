//! Two-state digital logic with an ideal clock.

use kernel::SimError;
use std::collections::BTreeMap;

/// Maximum settle iterations before a combinational loop is reported.
const MAX_SETTLE_ITERATIONS: usize = 10_000;

/// Identifies a digital net.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NetId(usize);

impl NetId {
    /// Returns the raw index.
    #[must_use]
    pub const fn raw(self) -> usize {
        self.0
    }
}

/// Identifies one ideal clock domain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockDomain(usize);

impl ClockDomain {
    /// Default clock domain.
    pub const DEFAULT: Self = Self(0);

    /// Creates a clock domain identifier.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the domain index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Identifies one reset domain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResetDomain(usize);

impl ResetDomain {
    /// Default reset domain.
    pub const DEFAULT: Self = Self(0);

    /// Creates a reset domain identifier.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the domain index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

mod builder;
mod components;
mod evaluate;
mod machine;
#[cfg(test)]
mod tests;
mod topology;
mod types;

pub use builder::DigitalBuilder;
use components::*;
use evaluate::*;
pub use machine::{CircuitStats, DigitalCircuit};
use types::{Dff, Gate};
