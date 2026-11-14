use super::*;

pub(crate) fn pins_exact(instance: &DeviceInstance, count: usize) -> Result<Vec<NodeId>, SimError> {
    let pins = instance.pins();
    if pins.len() != count {
        return Err(SimError::InvalidNetlist(format!(
            "{} requires {count} pins",
            instance.name()
        )));
    }
    Ok(pins.to_vec())
}

pub(crate) fn pins_three_or_four(instance: &DeviceInstance) -> Result<[NodeId; 4], SimError> {
    let pins = instance.pins();
    if pins.len() != 3 && pins.len() != 4 {
        return Err(SimError::InvalidNetlist(format!(
            "{} requires three or four pins",
            instance.name()
        )));
    }
    let body = if pins.len() == 4 { pins[3] } else { pins[2] };
    Ok([pins[0], pins[1], pins[2], body])
}

pub(crate) fn branch_index(ctx: &MnaContext, id: DeviceId) -> Result<usize, SimError> {
    ctx.branch_index(id)
        .ok_or_else(|| SimError::Internal(format!("missing branch for device {}", id.raw())))
}

pub(crate) fn stamp_conductance(
    ctx: &mut MnaContext,
    first: NodeId,
    second: NodeId,
    conductance: f64,
) -> Result<(), SimError> {
    let a = ctx.node_index(first);
    let b = ctx.node_index(second);
    ctx.add_matrix_opt(a, a, conductance)?;
    ctx.add_matrix_opt(a, b, -conductance)?;
    ctx.add_matrix_opt(b, a, -conductance)?;
    ctx.add_matrix_opt(b, b, conductance)?;
    Ok(())
}

pub(crate) fn stamp_capacitor(
    ctx: &mut MnaContext,
    first: NodeId,
    second: NodeId,
    capacitance: f64,
    voltage_prev: f64,
    dt: f64,
) -> Result<(), SimError> {
    if capacitance <= 0.0 {
        return Ok(());
    }
    let conductance = capacitance / dt;
    stamp_conductance(ctx, first, second, conductance)?;
    let a = ctx.node_index(first);
    let b = ctx.node_index(second);
    let current = conductance * voltage_prev;
    ctx.add_rhs_opt(a, current)?;
    ctx.add_rhs_opt(b, -current)
}

pub(crate) fn stamp_current_source(
    ctx: &mut MnaContext,
    from: NodeId,
    to: NodeId,
    current: f64,
) -> Result<(), SimError> {
    let from_index = ctx.node_index(from);
    let to_index = ctx.node_index(to);
    ctx.add_rhs_opt(from_index, -current)?;
    ctx.add_rhs_opt(to_index, current)?;
    Ok(())
}

pub(crate) fn stamp_voltage_branch(
    ctx: &mut MnaContext,
    id: DeviceId,
    positive: NodeId,
    negative: NodeId,
    voltage: f64,
) -> Result<(), SimError> {
    let branch = branch_index(ctx, id)?;
    let p = ctx.node_index(positive);
    let n = ctx.node_index(negative);
    ctx.add_matrix_opt(p, Some(branch), 1.0)?;
    ctx.add_matrix_opt(n, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), p, 1.0)?;
    ctx.add_matrix_opt(Some(branch), n, -1.0)?;
    ctx.add_rhs(branch, voltage)
}

pub(crate) fn stamp_inductor_branch(
    ctx: &mut MnaContext,
    id: DeviceId,
    positive: NodeId,
    negative: NodeId,
    resistance: f64,
    rhs: f64,
) -> Result<(), SimError> {
    let branch = branch_index(ctx, id)?;
    let p = ctx.node_index(positive);
    let n = ctx.node_index(negative);
    ctx.add_matrix_opt(p, Some(branch), 1.0)?;
    ctx.add_matrix_opt(n, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), p, 1.0)?;
    ctx.add_matrix_opt(Some(branch), n, -1.0)?;
    ctx.add_matrix(branch, branch, -resistance)?;
    ctx.add_rhs(branch, rhs)
}

/// Numerical derivative step for device linearization.
pub(crate) const DERIVATIVE_STEP: f64 = 1e-6;

pub(crate) fn finite_difference<F: Fn(f64) -> f64>(function: F, value: f64, step: f64) -> f64 {
    (function(value + step) - function(value)) / step
}

/// Linearized terminal current and its derivatives.
#[derive(Clone, Copy, Default)]
pub(crate) struct Linearized {
    pub(crate) current: f64,
    pub(crate) gm: f64,
    pub(crate) gds: f64,
    pub(crate) gms: f64,
    pub(crate) gmb: f64,
}

impl Linearized {
    pub(crate) fn from_three(current: f64, gm: f64, gds: f64, gms: f64) -> Self {
        Self {
            current,
            gm,
            gds,
            gms,
            gmb: 0.0,
        }
    }
}

pub(crate) fn stamp_three_terminal(
    ctx: &mut MnaContext,
    pins: [NodeId; 3],
    current: f64,
    gm: f64,
    gds: f64,
    gms: f64,
    voltages: (f64, f64, f64),
) -> Result<(), SimError> {
    let (vg, vd, vs) = voltages;
    let state = Linearized::from_three(current, gm, gds, gms);
    stamp_four_terminal(
        ctx,
        [pins[0], pins[1], pins[2], pins[2]],
        state,
        (vg, vd, vs, vs),
    )
}

pub(crate) fn stamp_four_terminal(
    ctx: &mut MnaContext,
    pins: [NodeId; 4],
    state: Linearized,
    voltages: (f64, f64, f64, f64),
) -> Result<(), SimError> {
    let gate = ctx.node_index(pins[0]);
    let drain = ctx.node_index(pins[1]);
    let source = ctx.node_index(pins[2]);
    let body = ctx.node_index(pins[3]);
    let (vg, vd, vs, vb) = voltages;
    let constant = state.gm * vg + state.gds * vd + state.gms * vs + state.gmb * vb - state.current;
    let cols = (gate, drain, source, body);
    let values = (state.gm, state.gds, state.gms, state.gmb);
    let negative = (-values.0, -values.1, -values.2, -values.3);
    stamp_row(ctx, drain, cols, values, constant)?;
    stamp_row(ctx, source, cols, negative, -constant)
}

fn stamp_row(
    ctx: &mut MnaContext,
    row: Option<usize>,
    cols: (Option<usize>, Option<usize>, Option<usize>, Option<usize>),
    values: (f64, f64, f64, f64),
    constant: f64,
) -> Result<(), SimError> {
    ctx.add_matrix_opt(row, cols.0, values.0)?;
    ctx.add_matrix_opt(row, cols.1, values.1)?;
    ctx.add_matrix_opt(row, cols.2, values.2)?;
    ctx.add_matrix_opt(row, cols.3, values.3)?;
    ctx.add_rhs_opt(row, constant)
}
