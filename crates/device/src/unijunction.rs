//! Behavioral unijunction and programmable unijunction transistors.

use crate::model::{
    DeviceModel, non_negative_parameter, parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_current_source,
};
use kernel::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};
use std::collections::BTreeMap;

/// Trigger style of a unijunction device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnijunctionMode {
    Ujt,
    Put,
}

/// A three-terminal unijunction device with a latching low-resistance state.
pub struct UnijunctionModel {
    mode: UnijunctionMode,
    base_resistance: f64,
    eta: f64,
    vd: f64,
    ron: f64,
    iv: f64,
    leakage: f64,
    on: bool,
    current: f64,
    pins: [NodeId; 3],
}

impl UnijunctionModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_exact(instance, 3)?;
        let mode = mode_for(instance.kind())?;
        let params = instance.parameters();
        let (base_resistance, eta) = base_parameters(mode, params)?;
        let vd = non_negative_parameter(params, "vd", 0.7)?;
        let ron = positive_parameter(params, "ron", Some(10.0))?;
        let iv = non_negative_parameter(params, "iv", 1e-3)?;
        let leakage = non_negative_parameter(params, "leakage", 1e-9)?;
        Ok(Self {
            mode,
            base_resistance,
            eta,
            vd,
            ron,
            iv,
            leakage,
            on: false,
            current: 0.0,
            pins: [pins[0], pins[1], pins[2]],
        })
    }

    fn voltages(&self, solution: &Solution) -> [f64; 3] {
        [
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
        ]
    }

    fn triggers(&self, voltages: [f64; 3]) -> bool {
        match self.mode {
            UnijunctionMode::Ujt => self.ujt_triggers(voltages),
            UnijunctionMode::Put => self.put_triggers(voltages),
        }
    }

    fn ujt_triggers(&self, voltages: [f64; 3]) -> bool {
        let vbb = voltages[1] - voltages[0];
        if vbb <= 0.0 {
            return false;
        }
        voltages[2] - voltages[0] > self.eta * vbb + self.vd
    }

    fn put_triggers(&self, voltages: [f64; 3]) -> bool {
        voltages[0] > voltages[2] && voltages[0] - voltages[1] > self.vd
    }

    fn stamp_base(&self, ctx: &mut MnaContext) -> Result<(), SimError> {
        match self.mode {
            UnijunctionMode::Ujt => {
                stamp_conductance(ctx, self.pins[1], self.pins[0], 1.0 / self.base_resistance)
            }
            UnijunctionMode::Put => {
                stamp_conductance(ctx, self.pins[1], self.pins[2], 1.0 / self.base_resistance)
            }
        }
    }

    fn stamp_switch(&self, ctx: &mut MnaContext, voltages: [f64; 3]) -> Result<(), SimError> {
        let (positive, negative) = switch_pins(self.mode, self.pins);
        let voltage = switch_voltage(self.mode, voltages);
        if !self.on || voltage <= self.vd {
            return stamp_conductance(ctx, positive, negative, self.leakage);
        }
        stamp_conductance(ctx, positive, negative, 1.0 / self.ron)?;
        stamp_current_source(ctx, positive, negative, -self.vd / self.ron)
    }

    fn switch_current(&self, voltage: f64) -> f64 {
        if !self.on || voltage <= self.vd {
            return self.leakage * voltage;
        }
        (voltage - self.vd) / self.ron
    }
}

impl DeviceModel for UnijunctionModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltages = self.voltages(solution);
        if !self.on && self.triggers(voltages) {
            self.on = true;
        }
        self.stamp_base(ctx)?;
        self.stamp_switch(ctx, voltages)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let voltages = self.voltages(solution);
        let voltage = switch_voltage(self.mode, voltages);
        self.current = self.switch_current(voltage);
        if self.on && self.current < self.iv {
            self.on = false;
        }
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }
}

fn mode_for(kind: &DeviceKind) -> Result<UnijunctionMode, SimError> {
    match kind {
        DeviceKind::Ujt => Ok(UnijunctionMode::Ujt),
        DeviceKind::Put => Ok(UnijunctionMode::Put),
        _ => Err(SimError::Internal("unijunction kind mismatch".to_string())),
    }
}

fn base_parameters(
    mode: UnijunctionMode,
    params: &BTreeMap<String, f64>,
) -> Result<(f64, f64), SimError> {
    match mode {
        UnijunctionMode::Ujt => Ok((
            positive_parameter(params, "rb", Some(5000.0))?,
            eta_value(params)?,
        )),
        UnijunctionMode::Put => Ok((positive_parameter(params, "rg", Some(1e6))?, 0.0)),
    }
}

fn eta_value(params: &BTreeMap<String, f64>) -> Result<f64, SimError> {
    let eta = parameter(params, "eta", 0.6);
    if !(0.0..1.0).contains(&eta) {
        return Err(SimError::InvalidNetlist(
            "ujt eta must be between 0 and 1".to_string(),
        ));
    }
    Ok(eta)
}

fn switch_pins(mode: UnijunctionMode, pins: [NodeId; 3]) -> (NodeId, NodeId) {
    match mode {
        UnijunctionMode::Ujt => (pins[2], pins[0]),
        UnijunctionMode::Put => (pins[0], pins[2]),
    }
}

fn switch_voltage(mode: UnijunctionMode, voltages: [f64; 3]) -> f64 {
    match mode {
        UnijunctionMode::Ujt => voltages[2] - voltages[0],
        UnijunctionMode::Put => voltages[0] - voltages[2],
    }
}

#[cfg(test)]
mod tests {
    use super::{UnijunctionMode, UnijunctionModel};
    use kernel::{DeviceId, DeviceInstance, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        DeviceInstance::new(
            DeviceId::new(0),
            "q1",
            kind,
            vec![NodeId::new(1), NodeId::new(2), NodeId::GROUND],
            parameters,
        )
    }

    #[test]
    fn ujt_triggers_above_standoff_voltage() {
        let model =
            UnijunctionModel::from_instance(&instance(DeviceKind::Ujt, &[])).expect("model");
        assert_eq!(model.mode, UnijunctionMode::Ujt);
        assert!(!model.triggers([0.0, 10.0, 5.0]));
        assert!(model.triggers([0.0, 10.0, 7.0]));
    }

    #[test]
    fn put_triggers_above_gate_voltage() {
        let model =
            UnijunctionModel::from_instance(&instance(DeviceKind::Put, &[])).expect("model");
        assert!(!model.triggers([4.0, 4.0, 0.0]));
        assert!(model.triggers([5.0, 4.0, 0.0]));
    }

    #[test]
    fn out_of_range_eta_is_rejected() {
        assert!(
            UnijunctionModel::from_instance(&instance(DeviceKind::Ujt, &[("eta", 1.5)])).is_err()
        );
    }
}
