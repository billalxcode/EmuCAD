use super::*;

pub struct LogicUnit {
    /// Result bits, least significant first.
    pub result: Vec<NetId>,
    /// Result is zero.
    pub zero: NetId,
    /// Result bit 7.
    pub negative: NetId,
    /// Signed result sign. Overflow is always zero for logic operations.
    pub sign: NetId,
}

/// Builds the 8-bit AVR logic unit.
///
/// `op` selects AND, OR, XOR, and NOT in that order.
pub fn logic_unit(
    builder: &mut DigitalBuilder,
    first: &[NetId],
    second: &[NetId],
    op: &[NetId],
) -> Result<LogicUnit, SimError> {
    validate_logic(first, second, op)?;
    let result = (0..8)
        .map(|bit| logic_bit(builder, first[bit], second[bit], op))
        .collect::<Result<Vec<_>, _>>()?;
    let any = or_all(builder, &result);
    let negative = result[7];
    Ok(LogicUnit {
        result,
        zero: builder.not(any),
        negative,
        sign: negative,
    })
}

fn validate_logic(first: &[NetId], second: &[NetId], op: &[NetId]) -> Result<(), SimError> {
    if first.len() != 8 || second.len() != 8 || op.len() != 2 {
        return Err(SimError::InvalidNetlist(
            "AVR logic expects 8-bit operands and 2-bit opcode".to_string(),
        ));
    }
    Ok(())
}

fn logic_bit(
    builder: &mut DigitalBuilder,
    first: NetId,
    second: NetId,
    op: &[NetId],
) -> Result<NetId, SimError> {
    let options = [
        builder.and(first, second),
        builder.or(first, second),
        builder.xor(first, second),
        builder.not(first),
    ];
    mux_tree(builder, &options, op)
}
