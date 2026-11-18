//! Behavioral RF amplifier modules.

use crate::controlled::VcvsModel;
use crate::model::{DeviceModel, pins_exact, positive_parameter, stamp_conductance};
use kernel::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};

/// A three-pin RF amplifier using an ideal voltage-controlled source.
pub struct RfAmplifierModel {
    inner: VcvsModel,
    input_resistance: f64,
    pins: [NodeId; 3],
}

impl RfAmplifierModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let input_resistance =
            positive_parameter(instance.parameters(), "input_resistance", Some(1e6))?;
        let vcvs = DeviceInstance::new(
            instance.id(),
            instance.name().to_string(),
            DeviceKind::Vcvs,
            vec![pins[1], pins[2], pins[0], pins[2]],
            instance.parameters().clone(),
        );
        Ok(Self {
            inner: VcvsModel::from_instance(&vcvs)?,
            input_resistance,
            pins: [pins[0], pins[1], pins[2]],
        })
    }
}

impl DeviceModel for RfAmplifierModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_conductance(
            ctx,
            self.pins[0],
            self.pins[2],
            1.0 / self.input_resistance.max(1e-9),
        )?;
        self.inner.stamp(ctx, solution, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        self.inner.update(solution)
    }
}

#[cfg(test)]
mod tests {
    use super::RfAmplifierModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "A1".to_string(),
            DeviceKind::RfAmplifier,
            vec![NodeId::new(1), NodeId::new(2), NodeId::GROUND],
            parameters,
        )
    }

    #[test]
    fn rf_amplifier_builds_from_three_pins() {
        let model = RfAmplifierModel::from_instance(&instance(&[("gain", 10.0)]));
        assert!(model.is_ok());
    }
}
