//! Tri-state bus resolution for digital circuits.

use super::blocks::or_all;
use super::circuit::{DigitalBuilder, NetId};
use super::vector::DigitalVector;
use kernel::SimError;

/// Pull level used when no bus driver is enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BusPull {
    /// Pull the bus low.
    Low,
    /// Pull the bus high.
    High,
}

/// One tri-state bus driver.
pub struct BusDriver {
    /// Data driven when enabled.
    pub data: DigitalVector,
    /// Driver enable.
    pub enable: NetId,
}

/// Resolved bus value and contention flag.
pub struct BusResolver {
    /// Resolved bus value.
    pub value: DigitalVector,
    /// True when enabled drivers disagree.
    pub contention: NetId,
}

/// Resolves multiple tri-state drivers into one bus.
pub fn resolve_bus(
    builder: &mut DigitalBuilder,
    drivers: &[BusDriver],
    pull: BusPull,
) -> Result<BusResolver, SimError> {
    let width = validate_drivers(drivers)?;
    let (bits, contention) = resolve_bits(builder, drivers, pull, width);
    Ok(BusResolver {
        value: DigitalVector::new(bits)?,
        contention,
    })
}

fn validate_drivers(drivers: &[BusDriver]) -> Result<usize, SimError> {
    let Some(first) = drivers.first() else {
        return Err(SimError::InvalidNetlist(
            "bus needs at least one driver".to_string(),
        ));
    };
    let width = first.data.width();
    if drivers.iter().any(|driver| driver.data.width() != width) {
        return Err(SimError::InvalidNetlist(
            "bus driver width mismatch".to_string(),
        ));
    }
    Ok(width)
}

fn resolve_bits(
    builder: &mut DigitalBuilder,
    drivers: &[BusDriver],
    pull: BusPull,
    width: usize,
) -> (Vec<NetId>, NetId) {
    let mut bits = Vec::with_capacity(width);
    let mut contention = builder.constant(false);
    for index in 0..width {
        let (bit, conflict) = resolve_bit(builder, drivers, index, pull);
        bits.push(bit);
        contention = builder.or(contention, conflict);
    }
    (bits, contention)
}

fn resolve_bit(
    builder: &mut DigitalBuilder,
    drivers: &[BusDriver],
    index: usize,
    pull: BusPull,
) -> (NetId, NetId) {
    let enable: Vec<NetId> = drivers.iter().map(|driver| driver.enable).collect();
    let active = or_all(builder, &enable);
    let terms: Vec<NetId> = drivers
        .iter()
        .map(|driver| builder.and(driver.enable, driver.data.bits()[index]))
        .collect();
    let driven = or_all(builder, &terms);
    let idle = builder.constant(pull == BusPull::High);
    let value = builder.mux(idle, driven, active);
    (value, conflict_bit(builder, drivers, index))
}

fn conflict_bit(builder: &mut DigitalBuilder, drivers: &[BusDriver], index: usize) -> NetId {
    let mut conflict = builder.constant(false);
    for first in 0..drivers.len() {
        for second in (first + 1)..drivers.len() {
            let both = builder.and(drivers[first].enable, drivers[second].enable);
            let left = drivers[first].data.bits()[index];
            let right = drivers[second].data.bits()[index];
            let different = builder.xor(left, right);
            let pair = builder.and(both, different);
            conflict = builder.or(conflict, pair);
        }
    }
    conflict
}

#[cfg(test)]
mod tests {
    use super::{BusDriver, BusPull, resolve_bus};
    use crate::{DigitalBuilder, DigitalCircuit, DigitalVector};

    fn inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> DigitalVector {
        let bits = (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect();
        DigitalVector::new(bits).expect("vector")
    }

    fn expose(builder: &mut DigitalBuilder, prefix: &str, vector: &DigitalVector) {
        for (index, bit) in vector.bits().iter().enumerate() {
            builder.output(&format!("{prefix}{index}"), *bit);
        }
    }

    fn two_drivers(pull: BusPull) -> DigitalCircuit {
        let mut builder = DigitalBuilder::new();
        let drivers = [driver(&mut builder, "a"), driver(&mut builder, "b")];
        let bus = resolve_bus(&mut builder, &drivers, pull).expect("bus");
        expose(&mut builder, "y", &bus.value);
        builder.output("conflict", bus.contention);
        builder.build()
    }

    fn driver(builder: &mut DigitalBuilder, prefix: &str) -> BusDriver {
        BusDriver {
            data: inputs(builder, prefix, 1),
            enable: builder.input(&format!("e{prefix}")),
        }
    }

    #[test]
    fn single_driver_and_pull_low() {
        let mut builder = DigitalBuilder::new();
        let data = inputs(&mut builder, "d", 2);
        let enable = builder.input("en");
        let bus =
            resolve_bus(&mut builder, &[BusDriver { data, enable }], BusPull::Low).expect("bus");
        expose(&mut builder, "y", &bus.value);
        builder.output("conflict", bus.contention);
        let mut circuit = builder.build();
        circuit.set_input("en", true).expect("en");
        circuit.set_input("d0", true).expect("d0");
        circuit.settle().expect("settle");
        assert_eq!(circuit.output("y0"), Some(true));
        assert_eq!(circuit.output("y1"), Some(false));
        circuit.set_input("en", false).expect("en");
        circuit.settle().expect("settle");
        assert_eq!(circuit.output("y0"), Some(false));
        assert_eq!(circuit.output("y1"), Some(false));
    }

    #[test]
    fn equal_enabled_drivers_have_no_contention() {
        let mut circuit = two_drivers(BusPull::Low);
        circuit.set_input("ea", true).expect("ea");
        circuit.set_input("eb", true).expect("eb");
        circuit.set_input("a0", true).expect("a0");
        circuit.set_input("b0", true).expect("b0");
        circuit.settle().expect("settle");
        assert_eq!(circuit.output("y0"), Some(true));
        assert_eq!(circuit.output("conflict"), Some(false));
    }

    #[test]
    fn conflicting_drivers_set_contention() {
        let mut circuit = two_drivers(BusPull::High);
        circuit.set_input("ea", true).expect("ea");
        circuit.set_input("eb", true).expect("eb");
        circuit.set_input("a0", true).expect("a0");
        circuit.set_input("b0", false).expect("b0");
        circuit.settle().expect("settle");
        assert_eq!(circuit.output("conflict"), Some(true));
    }

    #[test]
    fn rejects_empty_or_mismatched_drivers() {
        let mut builder = DigitalBuilder::new();
        assert!(resolve_bus(&mut builder, &[], BusPull::Low).is_err());
        let (short, long) = (inputs(&mut builder, "a", 1), inputs(&mut builder, "b", 2));
        let enable = builder.input("en");
        assert!(
            resolve_bus(
                &mut builder,
                &[
                    BusDriver {
                        data: short,
                        enable,
                    },
                    BusDriver { data: long, enable },
                ],
                BusPull::Low,
            )
            .is_err()
        );
    }
}
