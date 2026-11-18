//! Composite diode arrays built from the existing diode model.

use crate::diode::DiodeModel;
use crate::model::{
    DeviceModel, non_negative_parameter, pins_exact, positive_parameter, thermal_voltage,
};
use kernel::{
    Device, DeviceId, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// A four-diode bridge with pins AC1, AC2, positive, negative.
pub struct DiodeBridgeModel {
    diodes: [DiodeModel; 4],
}

/// Two independent diodes with pins A1, K1, A2, K2.
pub struct DualDiodeModel {
    diodes: [DiodeModel; 2],
}

impl DiodeBridgeModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        let diodes = [
            body_diode(params, [pins[0], pins[2]], "d1")?,
            body_diode(params, [pins[1], pins[2]], "d2")?,
            body_diode(params, [pins[3], pins[0]], "d3")?,
            body_diode(params, [pins[3], pins[1]], "d4")?,
        ];
        Ok(Self { diodes })
    }
}

impl DualDiodeModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 4)?;
        let params = instance.parameters();
        let diodes = [
            body_diode(params, [pins[0], pins[1]], "d1")?,
            body_diode(params, [pins[2], pins[3]], "d2")?,
        ];
        Ok(Self { diodes })
    }
}

fn body_diode(
    params: &BTreeMap<String, f64>,
    pins: [NodeId; 2],
    name: &str,
) -> Result<DiodeModel, SimError> {
    let instance = DeviceInstance::new(
        DeviceId::new(0),
        name,
        DeviceKind::Diode,
        vec![pins[0], pins[1]],
        diode_parameters(params)?,
    );
    DiodeModel::from_instance(&instance)
}

fn diode_parameters(params: &BTreeMap<String, f64>) -> Result<BTreeMap<String, f64>, SimError> {
    let forward = positive_parameter(params, "forward_voltage", Some(0.7))?;
    let emission = positive_parameter(params, "n", Some(1.0))?;
    let resistance = non_negative_parameter(params, "rs", 0.01)?;
    let vt = thermal_voltage(params)?;
    let saturation = (-forward / (emission * vt)).exp();
    if !saturation.is_finite() || saturation <= 0.0 {
        return Err(SimError::InvalidNetlist(
            "diode array forward voltage is out of range".to_string(),
        ));
    }
    let mut diode = BTreeMap::new();
    diode.insert("is".to_string(), saturation);
    diode.insert("n".to_string(), emission);
    diode.insert("rs".to_string(), resistance);
    Ok(diode)
}

fn stamp_diodes(
    diodes: &mut [DiodeModel],
    ctx: &mut MnaContext,
    solution: &Solution,
    dt: Option<f64>,
) -> Result<(), SimError> {
    for diode in diodes {
        diode.stamp(ctx, solution, dt)?;
    }
    Ok(())
}

fn update_diodes(diodes: &mut [DiodeModel], solution: &Solution) -> Result<(), SimError> {
    for diode in diodes {
        diode.update(solution)?;
    }
    Ok(())
}

impl DeviceModel for DiodeBridgeModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_diodes(&mut self.diodes, ctx, solution, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        update_diodes(&mut self.diodes, solution)
    }
}

impl DeviceModel for DualDiodeModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        stamp_diodes(&mut self.diodes, ctx, solution, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        update_diodes(&mut self.diodes, solution)
    }
}

#[cfg(test)]
mod tests {
    use super::{DiodeBridgeModel, DualDiodeModel, diode_parameters};
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "d1",
            kind,
            vec![
                NodeId::new(1),
                NodeId::new(2),
                NodeId::new(3),
                NodeId::new(4),
            ],
            parameters,
        )
    }

    #[test]
    fn lower_forward_voltage_raises_saturation_current() {
        let silicon = diode_parameters(&BTreeMap::from([("forward_voltage".to_string(), 0.7)]))
            .expect("silicon");
        let schottky = diode_parameters(&BTreeMap::from([("forward_voltage".to_string(), 0.3)]))
            .expect("schottky");
        assert!(schottky["is"] > silicon["is"]);
    }

    #[test]
    fn bridge_and_dual_build_from_four_pins() {
        assert!(DiodeBridgeModel::from_instance(&instance(DeviceKind::DiodeBridge, &[])).is_ok());
        assert!(DualDiodeModel::from_instance(&instance(DeviceKind::DualDiode, &[])).is_ok());
    }

    #[test]
    fn invalid_forward_voltage_is_rejected() {
        assert!(
            DiodeBridgeModel::from_instance(&instance(
                DeviceKind::DiodeBridge,
                &[("forward_voltage", 0.0)],
            ))
            .is_err()
        );
    }
}
