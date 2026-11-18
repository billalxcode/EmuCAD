//! AVR arithmetic unit.

use cells::{DigitalBuilder, NetId, and_all, digital_full_adder, mux_tree, or_all};
use kernel::SimError;

mod arithmetic;
mod logic;
#[cfg(test)]
mod tests;
mod units;

pub use arithmetic::{ArithmeticUnit, arithmetic_unit};
pub use logic::{LogicUnit, logic_unit};
pub use units::{CompareUnit, compare_unit, decrement_unit, increment_unit, negate_unit};

/// Selects between a register operand and an immediate operand.
pub fn select_operand(
    builder: &mut DigitalBuilder,
    register: &[NetId],
    immediate: &[NetId],
    use_immediate: NetId,
) -> Result<Vec<NetId>, SimError> {
    if register.len() != immediate.len() {
        return Err(SimError::InvalidNetlist(
            "operand width mismatch".to_string(),
        ));
    }
    Ok(register
        .iter()
        .zip(immediate)
        .map(|(&register, &immediate)| builder.mux(register, immediate, use_immediate))
        .collect())
}
