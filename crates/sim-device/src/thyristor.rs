//! Behavioral thyristor family models with latching main terminals.

use crate::model::{
    DeviceModel, non_negative_parameter, parameter, pins_exact, positive_parameter,
    stamp_conductance, stamp_current_source,
};
use sim_core::{Device, DeviceInstance, DeviceKind, MnaContext, NodeId, SimError, Solution};
use std::collections::BTreeMap;

/// Behavior family within the thyristor model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThyristorKind {
    Scr,
    Triac,
    Diac,
    Gto,
    Scs,
}

struct ThyristorDefaults {
    vbo: f64,
    vgt: f64,
    vgt_off: f64,
    ron: f64,
    vd: f64,
}

struct ThyristorParams {
    vbo: f64,
    vgt: f64,
    vgt_off: f64,
    holding: f64,
    ron: f64,
    vd: f64,
    leakage: f64,
    photo: f64,
    photo_trigger: f64,
}

/// A two, three, or four terminal thyristor with a latching state.
pub struct ThyristorModel {
    kind: ThyristorKind,
    params: ThyristorParams,
    on: bool,
    current: f64,
    pins: Vec<NodeId>,
}

impl ThyristorModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let kind = kind_for(instance.kind())?;
        let pins = pins_for(kind, instance)?;
        let params = parameters(kind, instance.parameters())?;
        Ok(Self {
            kind,
            params,
            on: false,
            current: 0.0,
            pins,
        })
    }

    fn main_pins(&self) -> (NodeId, NodeId) {
        match self.kind {
            ThyristorKind::Diac => (self.pins[0], self.pins[1]),
            ThyristorKind::Scs => (self.pins[0], self.pins[3]),
            _ => (self.pins[0], self.pins[2]),
        }
    }

    fn node(&self, solution: &Solution, index: usize) -> f64 {
        solution.voltage(self.pins[index]).unwrap_or(0.0)
    }

    fn main_voltage(&self, solution: &Solution) -> f64 {
        let (positive, negative) = self.main_pins();
        solution.voltage(positive).unwrap_or(0.0) - solution.voltage(negative).unwrap_or(0.0)
    }

    fn gate_voltage(&self, solution: &Solution) -> f64 {
        self.node(solution, 1) - self.node(solution, 2)
    }

    fn triggers(&self, solution: &Solution) -> bool {
        match self.kind {
            ThyristorKind::Scr => self.scr_triggers(solution),
            ThyristorKind::Triac => self.triac_triggers(solution),
            ThyristorKind::Diac => self.main_voltage(solution).abs() > self.params.vbo,
            ThyristorKind::Gto => self.gate_voltage(solution) > self.params.vgt,
            ThyristorKind::Scs => self.scs_triggers(solution),
        }
    }

    fn scr_triggers(&self, solution: &Solution) -> bool {
        self.gate_voltage(solution) > self.params.vgt
            || self.params.photo >= self.params.photo_trigger
    }

    fn triac_triggers(&self, solution: &Solution) -> bool {
        self.gate_voltage(solution).abs() > self.params.vgt
    }

    fn scs_triggers(&self, solution: &Solution) -> bool {
        self.node(solution, 2) - self.node(solution, 3) > self.params.vgt
    }

    fn turns_off(&self, solution: &Solution) -> bool {
        match self.kind {
            ThyristorKind::Gto => self.gate_voltage(solution) < self.params.vgt_off,
            ThyristorKind::Scs => {
                self.node(solution, 1) - self.node(solution, 0) > self.params.vgt_off
            }
            _ => false,
        }
    }

    fn is_symmetric(&self) -> bool {
        matches!(self.kind, ThyristorKind::Triac | ThyristorKind::Diac)
    }

    fn below_offset(&self, voltage: f64) -> bool {
        if self.is_symmetric() {
            return voltage.abs() <= self.params.vd;
        }
        voltage <= self.params.vd
    }

    fn switch_current(&self, voltage: f64) -> f64 {
        if !self.on || self.below_offset(voltage) {
            return self.params.leakage * voltage;
        }
        if self.is_symmetric() {
            return voltage.signum() * (voltage.abs() - self.params.vd) / self.params.ron;
        }
        (voltage - self.params.vd) / self.params.ron
    }

    fn offset_current(&self, voltage: f64) -> f64 {
        if self.is_symmetric() {
            return -voltage.signum() * self.params.vd / self.params.ron;
        }
        -self.params.vd / self.params.ron
    }

    fn stamp_switch(&self, ctx: &mut MnaContext, voltage: f64) -> Result<(), SimError> {
        let (positive, negative) = self.main_pins();
        if !self.on || self.below_offset(voltage) {
            return stamp_conductance(ctx, positive, negative, self.params.leakage);
        }
        stamp_conductance(ctx, positive, negative, 1.0 / self.params.ron)?;
        stamp_current_source(ctx, positive, negative, self.offset_current(voltage))
    }
}

impl DeviceModel for ThyristorModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        _dt: Option<f64>,
    ) -> Result<(), SimError> {
        if !self.on && self.triggers(solution) {
            self.on = true;
        }
        if self.on && self.turns_off(solution) {
            self.on = false;
        }
        let voltage = self.main_voltage(solution);
        self.stamp_switch(ctx, voltage)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let voltage = self.main_voltage(solution);
        self.current = self.switch_current(voltage);
        if self.on && self.current.abs() < self.params.holding {
            self.on = false;
        }
        Ok(())
    }

    fn terminal_current(&self, _solution: &Solution) -> Option<f64> {
        Some(self.current)
    }
}

fn kind_for(kind: &DeviceKind) -> Result<ThyristorKind, SimError> {
    match kind {
        DeviceKind::Scr => Ok(ThyristorKind::Scr),
        DeviceKind::Triac => Ok(ThyristorKind::Triac),
        DeviceKind::Diac => Ok(ThyristorKind::Diac),
        DeviceKind::Gto => Ok(ThyristorKind::Gto),
        DeviceKind::Scs => Ok(ThyristorKind::Scs),
        _ => Err(SimError::Internal("thyristor kind mismatch".to_string())),
    }
}

fn pins_for(kind: ThyristorKind, instance: &DeviceInstance) -> Result<Vec<NodeId>, SimError> {
    match kind {
        ThyristorKind::Diac => pins_exact(instance, 2),
        ThyristorKind::Scs => pins_exact(instance, 4),
        _ => pins_exact(instance, 3),
    }
}

fn parameters(
    kind: ThyristorKind,
    params: &BTreeMap<String, f64>,
) -> Result<ThyristorParams, SimError> {
    let defaults = defaults(kind);
    Ok(ThyristorParams {
        vbo: finite_value(params, "vbo", defaults.vbo)?,
        vgt: finite_value(params, "vgt", defaults.vgt)?,
        vgt_off: finite_value(params, "vgt_off", defaults.vgt_off)?,
        holding: non_negative_parameter(params, "holding_current", 0.01)?,
        ron: positive_parameter(params, "ron", Some(defaults.ron))?,
        vd: non_negative_parameter(params, "vd", defaults.vd)?,
        leakage: non_negative_parameter(params, "leakage", 1e-9)?,
        photo: non_negative_parameter(params, "photo_current", 0.0)?,
        photo_trigger: positive_parameter(params, "photo_trigger", Some(1e-3))?,
    })
}

fn finite_value(params: &BTreeMap<String, f64>, name: &str, default: f64) -> Result<f64, SimError> {
    let value = parameter(params, name, default);
    if !value.is_finite() {
        return Err(SimError::InvalidNetlist(format!(
            "thyristor {name} must be finite"
        )));
    }
    Ok(value)
}

fn defaults(kind: ThyristorKind) -> ThyristorDefaults {
    match kind {
        ThyristorKind::Scr => thyristor_defaults(2.0, 0.0, 1.0, 0.7),
        ThyristorKind::Triac => thyristor_defaults(5.0, 0.0, 1.0, 1.0),
        ThyristorKind::Diac => thyristor_defaults(5.0, 0.0, 10.0, 1.0),
        ThyristorKind::Gto => thyristor_defaults(5.0, -5.0, 1.0, 0.7),
        ThyristorKind::Scs => thyristor_defaults(5.0, 1.0, 1.0, 0.7),
    }
}

const fn thyristor_defaults(vgt: f64, vgt_off: f64, ron: f64, vd: f64) -> ThyristorDefaults {
    ThyristorDefaults {
        vbo: 30.0,
        vgt,
        vgt_off,
        ron,
        vd,
    }
}

#[cfg(test)]
mod tests {
    use super::{ThyristorKind, ThyristorModel};
    use sim_core::{DeviceId, DeviceInstance, DeviceKind, NodeId, Solution};
    use std::collections::BTreeMap;

    fn instance(kind: DeviceKind, pins: usize, params: &[(&str, f64)]) -> DeviceInstance {
        let parameters: BTreeMap<_, _> = params
            .iter()
            .map(|(name, value)| ((*name).to_string(), *value))
            .collect();
        let pin_ids = (1..=pins)
            .map(|index| NodeId::new(u32::try_from(index).expect("pin index")))
            .collect();
        DeviceInstance::new(DeviceId::new(0), "q1", kind, pin_ids, parameters)
    }

    fn solution(values: &[(usize, f64)]) -> Solution {
        let mut solution = Solution::zero(5, 0);
        for (index, value) in values {
            solution.node_voltages[*index] = *value;
        }
        solution
    }

    #[test]
    fn scr_triggers_on_gate_voltage() {
        let model =
            ThyristorModel::from_instance(&instance(DeviceKind::Scr, 3, &[])).expect("model");
        assert!(!model.triggers(&solution(&[(2, 0.0), (3, 0.0)])));
        assert!(model.triggers(&solution(&[(2, 3.0), (3, 0.0)])));
    }

    #[test]
    fn lascr_triggers_on_photo_current() {
        let model = ThyristorModel::from_instance(&instance(
            DeviceKind::Scr,
            3,
            &[("photo_current", 1e-2), ("photo_trigger", 1e-3)],
        ))
        .expect("model");
        assert!(model.triggers(&solution(&[])));
    }

    #[test]
    fn diac_triggers_on_breakover() {
        let model =
            ThyristorModel::from_instance(&instance(DeviceKind::Diac, 2, &[])).expect("model");
        assert!(!model.triggers(&solution(&[(1, 20.0), (2, 0.0)])));
        assert!(model.triggers(&solution(&[(1, 40.0), (2, 0.0)])));
    }

    #[test]
    fn gto_turns_off_with_negative_gate() {
        let model =
            ThyristorModel::from_instance(&instance(DeviceKind::Gto, 3, &[])).expect("model");
        assert!(!model.turns_off(&solution(&[(2, 0.0), (3, 0.0)])));
        assert!(model.turns_off(&solution(&[(2, -6.0), (3, 0.0)])));
    }

    #[test]
    fn scs_turns_off_with_anode_gate() {
        let model =
            ThyristorModel::from_instance(&instance(DeviceKind::Scs, 4, &[])).expect("model");
        assert!(!model.turns_off(&solution(&[])));
        assert!(model.turns_off(&solution(&[(2, 6.0), (1, 0.0)])));
    }

    #[test]
    fn model_kinds_match_device_kinds() {
        assert_eq!(
            ThyristorModel::from_instance(&instance(DeviceKind::Triac, 3, &[]))
                .expect("model")
                .kind,
            ThyristorKind::Triac
        );
    }
}
