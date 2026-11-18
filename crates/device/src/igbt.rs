//! Behavioral insulated-gate bipolar transistor model.

use crate::diode::DiodeModel;
use crate::model::{
    DERIVATIVE_STEP, DeviceModel, finite_difference, non_negative_parameter, parameter, pins_exact,
    positive_parameter, stamp_three_terminal,
};
use kernel::{
    Device, DeviceId, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution,
};
use std::collections::BTreeMap;

/// A three-terminal IGBT with an antiparallel body diode.
pub struct IgbtModel {
    vth: f64,
    kp: f64,
    lambda: f64,
    knee: f64,
    body: DiodeModel,
    pins: [NodeId; 3],
}

impl IgbtModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let params = instance.parameters();
        let vth = parameter(params, "vth", 5.0);
        if !vth.is_finite() {
            return Err(SimError::InvalidNetlist(
                "igbt threshold must be finite".to_string(),
            ));
        }
        let kp = positive_parameter(params, "kp", Some(1.0))?;
        let lambda = non_negative_parameter(params, "lambda", 0.01)?;
        let knee = positive_parameter(params, "vce_sat", Some(0.5))?;
        let body = body_diode(pins[2], pins[0], params)?;
        Ok(Self {
            vth,
            kp,
            lambda,
            knee,
            body,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn channel_current(&self, vge: f64, vce: f64) -> f64 {
        let overdrive = vge - self.vth - self.knee;
        if overdrive <= 0.0 || vce <= 0.0 {
            return 0.0;
        }
        let lambda = 1.0 + self.lambda * vce;
        if vce < overdrive {
            self.kp * (overdrive * vce - 0.5 * vce * vce) * lambda
        } else {
            0.5 * self.kp * overdrive * overdrive * lambda
        }
    }

    fn linearize(&self, vg: f64, vc: f64, ve: f64) -> (f64, f64, f64, f64) {
        let current = self.channel_current(vg - ve, vc - ve);
        let gm = finite_difference(
            |value| self.channel_current(value - ve, vc - ve),
            vg,
            DERIVATIVE_STEP,
        );
        let gds = finite_difference(
            |value| self.channel_current(vg - ve, value - ve),
            vc,
            DERIVATIVE_STEP,
        );
        let gms = finite_difference(
            |value| self.channel_current(vg - value, vc - value),
            ve,
            DERIVATIVE_STEP,
        );
        (current, gm, gds, gms)
    }
}

fn body_diode(
    emitter: NodeId,
    collector: NodeId,
    params: &BTreeMap<String, f64>,
) -> Result<DiodeModel, SimError> {
    let instance = DeviceInstance::new(
        DeviceId::new(0),
        "body",
        DeviceKind::Diode,
        vec![emitter, collector],
        body_parameters(params),
    );
    DiodeModel::from_instance(&instance)
}

fn body_parameters(params: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
    let mut body = BTreeMap::new();
    body.insert("is".to_string(), parameter(params, "is", 1e-6));
    body.insert("n".to_string(), parameter(params, "n", 1.0));
    body.insert("rs".to_string(), parameter(params, "rs", 0.01));
    body
}

impl IgbtModel {
    fn voltages(&self, solution: &Solution) -> (f64, f64, f64) {
        (
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
        )
    }

    fn stamp_channel(
        &self,
        ctx: &mut MnaContext,
        vg: f64,
        vc: f64,
        ve: f64,
    ) -> Result<(), SimError> {
        let (current, gm, gds, gms) = self.linearize(vg, vc, ve);
        stamp_three_terminal(
            ctx,
            [self.pins[1], self.pins[0], self.pins[2]],
            current,
            gm,
            gds,
            gms,
            (vg, vc, ve),
        )
    }
}

impl DeviceModel for IgbtModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let (vg, vc, ve) = self.voltages(solution);
        self.stamp_channel(ctx, vg, vc, ve)?;
        self.body.stamp(ctx, solution, dt)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        self.body.update(solution)
    }
}

#[cfg(test)]
mod tests {
    use super::IgbtModel;
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "q1",
            DeviceKind::Igbt,
            vec![NodeId::new(1), NodeId::new(2), NodeId::GROUND],
            parameters,
        )
    }

    #[test]
    fn gate_voltage_controls_channel() {
        let model = IgbtModel::from_instance(&instance(&[])).expect("model");
        assert_eq!(model.channel_current(4.0, 1.0), 0.0);
        assert!(model.channel_current(10.0, 1.0) > 0.1);
    }

    #[test]
    fn knee_voltage_slows_turn_on() {
        let model = IgbtModel::from_instance(&instance(&[])).expect("model");
        let sharp = IgbtModel::from_instance(&instance(&[("vce_sat", 0.1)])).expect("model");
        assert!(sharp.channel_current(10.0, 0.1) > model.channel_current(10.0, 0.1));
    }

    #[test]
    fn invalid_kp_is_rejected() {
        assert!(IgbtModel::from_instance(&instance(&[("kp", 0.0)])).is_err());
    }
}
