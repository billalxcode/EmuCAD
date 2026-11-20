//! SoC and MCU specifications.
//!
//! This crate contains data-only specs and validation. It does not
//! instantiate circuits or depend on `cells`.

#![forbid(unsafe_code)]

pub mod spec;

pub use spec::{
    BusProtocol, BusSpec, ClockSpec, CpuSpec, McuSpec, MemoryKind, MemorySpec, PeripheralKind,
    PeripheralSpec, PinSpec,
};
