//! Behavioral power converter modules.

use crate::model::{
    DeviceModel, finite_parameter, pins_exact, positive_parameter, stamp_voltage_branch,
};
use sim_core::{Device, DeviceId, DeviceInstance, MnaContext, NodeId, SimError, Solution};

/// A regulated converter with input current scaled by efficiency.
pub struct ConverterModel {
    id: DeviceId,
    output_voltage: f64,
    rated_input_voltage: f64,
    efficiency: f64,
    branch: Option<usize>,
    pins: [NodeId; 4],
}

impl ConverterModel {
    pub(crate) fn from_four_pin(
        instance: &DeviceInstance,
        rated_input_voltage: f64,
        efficiency: f64,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        Self::new(
            instance,
            [pins[0], pins[1], pins[2], pins[3]],
            rated_input_voltage,
            efficiency,
        )
    }

    pub(crate) fn from_three_pin(
        instance: &DeviceInstance,
        rated_input_voltage: f64,
        efficiency: f64,
    ) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        Self::new(
            instance,
            [pins[0], pins[2], pins[1], pins[2]],
            rated_input_voltage,
            efficiency,
        )
    }

    fn new(
        instance: &DeviceInstance,
        pins: [NodeId; 4],
        rated_input_voltage: f64,
        efficiency: f64,
    ) -> Result<Self, SimError> {
        let params = instance.parameters();
        let efficiency = positive_parameter(params, "efficiency", Some(efficiency))?;
        if efficiency > 1.0 {
            return Err(SimError::InvalidNetlist(
                "converter efficiency must be at most one".to_string(),
            ));
        }
        Ok(Self {
            id: instance.id(),
            output_voltage: finite_parameter(params, "output_voltage", 5.0)?,
            rated_input_voltage: positive_parameter(
                params,
                "rated_input_voltage",
                Some(rated_input_voltage),
            )?,
            efficiency,
            branch: None,
            pins,
        })
    }

    fn conversion_ratio(&self) -> f64 {
        self.output_voltage / (self.rated_input_voltage * self.efficiency)
    }
}

impl DeviceModel for ConverterModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        _solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_voltage_branch(
            ctx,
            self.id,
            self.pins[2],
            self.pins[3],
            self.output_voltage,
        )?;
        let branch = crate::model::branch_index(ctx, self.id)?;
        self.branch = Some(branch);
        let ratio = self.conversion_ratio();
        let input_plus = ctx.node_index(self.pins[0]);
        let input_minus = ctx.node_index(self.pins[1]);
        ctx.add_matrix_opt(input_plus, Some(branch), -ratio)?;
        ctx.add_matrix_opt(input_minus, Some(branch), ratio)
    }

    fn update(&mut self, _solution: &Solution) -> Result<(), SimError> {
        Ok(())
    }

    fn terminal_current(&self, solution: &Solution) -> Option<f64> {
        solution.branch_current(self.branch?)
    }
}

#[cfg(test)]
mod tests {
    use super::ConverterModel;
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, pins: usize, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        let nodes = (0..pins).map(|_| NodeId::GROUND).collect();
        DeviceInstance::new(DeviceId::new(0), "U1".to_string(), kind, nodes, parameters)
    }

    #[test]
    fn four_pin_converter_builds() {
        let model = ConverterModel::from_four_pin(
            &instance(DeviceKind::DcDcConverter, 4, &[("efficiency", 0.85)]),
            12.0,
            0.85,
        );
        assert!(model.is_ok());
    }

    #[test]
    fn three_pin_converter_rejects_high_efficiency() {
        let model = ConverterModel::from_three_pin(
            &instance(DeviceKind::ChargePump, 3, &[("efficiency", 1.5)]),
            5.0,
            0.8,
        );
        assert!(model.is_err());
    }
}
