use super::*;

pub(super) fn evaluate(gate: Gate, values: &mut [bool]) -> bool {
    match gate {
        Gate::Const(value, output) => set_value(values, output, value),
        Gate::Buf(input, output) => set_value(values, output, values[input.raw()]),
        Gate::Not(input, output) => set_value(values, output, !values[input.raw()]),
        Gate::And(first, second, output) => evaluate_and(first, second, output, values),
        Gate::Or(first, second, output) => evaluate_or(first, second, output, values),
        Gate::Xor(first, second, output) => evaluate_xor(first, second, output, values),
        Gate::Nand(first, second, output) => evaluate_nand(first, second, output, values),
        Gate::Nor(first, second, output) => evaluate_nor(first, second, output, values),
        Gate::Mux {
            first,
            second,
            select,
            output,
        } => evaluate_mux(first, second, select, output, values),
    }
}

fn evaluate_and(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] && values[second.raw()])
}

fn evaluate_or(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] || values[second.raw()])
}

fn evaluate_xor(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(values, output, values[first.raw()] ^ values[second.raw()])
}

fn evaluate_nand(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(
        values,
        output,
        !(values[first.raw()] && values[second.raw()]),
    )
}

fn evaluate_nor(first: NetId, second: NetId, output: NetId, values: &mut [bool]) -> bool {
    set_value(
        values,
        output,
        !(values[first.raw()] || values[second.raw()]),
    )
}

fn evaluate_mux(
    first: NetId,
    second: NetId,
    select: NetId,
    output: NetId,
    values: &mut [bool],
) -> bool {
    let value = if values[select.raw()] {
        values[second.raw()]
    } else {
        values[first.raw()]
    };
    set_value(values, output, value)
}

fn set_value(values: &mut [bool], net: NetId, value: bool) -> bool {
    let slot = &mut values[net.raw()];
    if *slot == value {
        return false;
    }
    *slot = value;
    true
}
