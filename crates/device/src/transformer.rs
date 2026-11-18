//! Two-winding transformer with mutual inductance.

use crate::model::{
    DeviceModel, IntegrationMethod, branch_index, non_negative_parameter, parameter, pins_exact,
    positive_parameter,
};
use kernel::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A four-terminal transformer with primary and secondary windings.
pub struct TransformerModel {
    id: DeviceId,
    primary_inductance: f64,
    secondary_inductance: f64,
    mutual: f64,
    primary_resistance: f64,
    secondary_resistance: f64,
    primary_prev: f64,
    secondary_prev: f64,
    branch: Option<usize>,
    pins: [NodeId; 4],
}

impl TransformerModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let ratio = positive_parameter(instance.parameters(), "turns_ratio", Some(1.0))?;
        let primary = positive_parameter(instance.parameters(), "inductance", Some(1e-3))?;
        let coupling = parameter(instance.parameters(), "coupling", 0.99);
        if !coupling.is_finite() || !(0.0..=1.0).contains(&coupling) {
            return Err(SimError::InvalidNetlist(
                "transformer coupling must be between 0 and 1".to_string(),
            ));
        }
        let primary_resistance = non_negative_parameter(instance.parameters(), "rp", 0.1)?;
        let default_secondary = primary_resistance * ratio * ratio;
        let secondary_resistance =
            non_negative_parameter(instance.parameters(), "rs", default_secondary)?;
        let secondary = primary * ratio * ratio;
        let mutual = coupling * primary * ratio;
        Ok(Self {
            id: instance.id(),
            primary_inductance: primary,
            secondary_inductance: secondary,
            mutual,
            primary_resistance,
            secondary_resistance,
            primary_prev: 0.0,
            secondary_prev: 0.0,
            branch: None,
            pins: [pins[0], pins[1], pins[2], pins[3]],
        })
    }
}

impl DeviceModel for TransformerModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        self.stamp_method(ctx, solution, dt, IntegrationMethod::BackwardEuler)
    }

    fn stamp_method(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        dt: Option<f64>,
        method: IntegrationMethod,
    ) -> Result<(), SimError> {
        let Some(dt) = dt else {
            return self.stamp_dc(ctx);
        };
        if method != IntegrationMethod::BackwardEuler {
            return Err(SimError::UnsupportedDevice(
                "transformer supports backward Euler only".to_string(),
            ));
        }
        self.stamp_transient(ctx, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let Some(primary) = self.branch else {
            return Ok(());
        };
        self.primary_prev = solution.branch_current(primary).unwrap_or(0.0);
        self.secondary_prev = solution.branch_current(primary + 1).unwrap_or(0.0);
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }
}

impl TransformerModel {
    fn stamp_transient(&mut self, ctx: &mut MnaContext, dt: f64) -> Result<(), SimError> {
        let primary = branch_index(ctx, self.id)?;
        let secondary = primary + 1;
        let pair = self.winding_pair(primary, secondary, dt);
        self.stamp_windings(ctx, &pair)?;
        self.branch = Some(primary);
        Ok(())
    }

    fn winding_pair(&self, primary: usize, secondary: usize, dt: f64) -> WindingPair {
        let a = self.primary_inductance / dt;
        let b = self.mutual / dt;
        let c = self.secondary_inductance / dt;
        WindingPair {
            primary,
            secondary,
            a,
            b,
            c,
            primary_rhs: -(a * self.primary_prev + b * self.secondary_prev),
            secondary_rhs: -(b * self.primary_prev + c * self.secondary_prev),
        }
    }

    fn stamp_windings(&self, ctx: &mut MnaContext, pair: &WindingPair) -> Result<(), SimError> {
        stamp_winding(
            ctx,
            pair.primary,
            pair.secondary,
            [self.pins[0], self.pins[1]],
            self.primary_resistance + pair.a,
            pair.b,
            pair.primary_rhs,
        )?;
        stamp_winding(
            ctx,
            pair.secondary,
            pair.primary,
            [self.pins[2], self.pins[3]],
            self.secondary_resistance + pair.c,
            pair.b,
            pair.secondary_rhs,
        )
    }

    fn stamp_dc(&mut self, ctx: &mut MnaContext) -> Result<(), SimError> {
        let primary = branch_index(ctx, self.id)?;
        let secondary = primary + 1;
        stamp_dc_winding(
            ctx,
            primary,
            secondary,
            [self.pins[0], self.pins[1]],
            self.primary_resistance,
        )?;
        stamp_dc_winding(
            ctx,
            secondary,
            primary,
            [self.pins[2], self.pins[3]],
            self.secondary_resistance,
        )?;
        self.branch = Some(primary);
        Ok(())
    }
}

struct WindingPair {
    primary: usize,
    secondary: usize,
    a: f64,
    b: f64,
    c: f64,
    primary_rhs: f64,
    secondary_rhs: f64,
}

fn stamp_dc_winding(
    ctx: &mut MnaContext,
    branch: usize,
    other: usize,
    pins: [NodeId; 2],
    resistance: f64,
) -> Result<(), SimError> {
    stamp_winding(ctx, branch, other, pins, resistance, 0.0, 0.0)
}

fn stamp_winding(
    ctx: &mut MnaContext,
    branch: usize,
    other: usize,
    pins: [NodeId; 2],
    self_coefficient: f64,
    mutual_coefficient: f64,
    rhs: f64,
) -> Result<(), SimError> {
    let positive = ctx.node_index(pins[0]);
    let negative = ctx.node_index(pins[1]);
    ctx.add_matrix_opt(positive, Some(branch), 1.0)?;
    ctx.add_matrix_opt(negative, Some(branch), -1.0)?;
    ctx.add_matrix_opt(Some(branch), positive, 1.0)?;
    ctx.add_matrix_opt(Some(branch), negative, -1.0)?;
    ctx.add_matrix_opt(Some(branch), Some(branch), -self_coefficient)?;
    ctx.add_matrix_opt(Some(branch), Some(other), -mutual_coefficient)?;
    ctx.add_rhs(branch, rhs)
}

#[cfg(test)]
mod tests {
    use super::TransformerModel;
    use crate::model::DeviceModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "t1",
            DeviceKind::Transformer,
            vec![
                NodeId::new(1),
                NodeId::GROUND,
                NodeId::new(2),
                NodeId::GROUND,
            ],
            parameters,
        )
    }

    #[test]
    fn turns_ratio_sets_secondary_inductance() {
        let model = TransformerModel::from_instance(&instance(&[
            ("turns_ratio", 2.0),
            ("inductance", 1e-3),
            ("coupling", 1.0),
        ]))
        .expect("model");
        assert!((model.secondary_inductance - 4e-3).abs() < 1e-12);
        assert!((model.mutual - 2e-3).abs() < 1e-12);
    }

    #[test]
    fn coupling_is_bounded() {
        let error = TransformerModel::from_instance(&instance(&[("coupling", 1.5)]));
        assert!(error.is_err());
    }

    #[test]
    fn terminal_current_needs_a_branch_sample() {
        let model = TransformerModel::from_instance(&instance(&[])).expect("model");
        assert!(
            model
                .terminal_current(&kernel::Solution::zero(3, 3))
                .is_none()
        );
    }
}
