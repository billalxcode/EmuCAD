use super::*;

pub struct MosfetModel {
    pub(super) kind: DeviceKind,
    pub(super) params: MosfetParams,
    pub(super) pins: [NodeId; 4],
    pub(super) vgs_prev: f64,
    pub(super) vgd_prev: f64,
    pub(super) vgb_prev: f64,
}

impl MosfetModel {
    pub(crate) fn from_instance(instance: &DeviceInstance) -> Result<Self, SimError> {
        let pins = pins_three_or_four(instance)?;
        let params = MosfetParams::from_map(instance.parameters())?;
        Ok(Self {
            kind: instance.kind().clone(),
            params,
            pins,
            vgs_prev: 0.0,
            vgd_prev: 0.0,
            vgb_prev: 0.0,
        })
    }

    fn sign(&self) -> f64 {
        if matches!(self.kind, DeviceKind::Pmos) {
            -1.0
        } else {
            1.0
        }
    }

    fn current(&self, vg: f64, vd: f64, vs: f64, vb: f64) -> f64 {
        let vbs = self.sign() * (vb - vs);
        match self.kind {
            DeviceKind::Nmos => nmos_current_body(vg - vs, vd - vs, vbs, self.params),
            DeviceKind::Pmos => -nmos_current_body(vs - vg, vs - vd, vbs, self.params),
            _ => 0.0,
        }
    }

    pub(super) fn linearization(&self, vg: f64, vd: f64, vs: f64, vb: f64) -> Linearized {
        let current = self.current(vg, vd, vs, vb);
        let gm = finite_difference(|value| self.current(value, vd, vs, vb), vg, DERIVATIVE_STEP);
        let gds = finite_difference(|value| self.current(vg, value, vs, vb), vd, DERIVATIVE_STEP);
        let gms = finite_difference(|value| self.current(vg, vd, value, vb), vs, DERIVATIVE_STEP);
        let gmb = finite_difference(|value| self.current(vg, vd, vs, value), vb, DERIVATIVE_STEP);
        Linearized {
            current,
            gm,
            gds,
            gms,
            gmb,
        }
    }

    fn voltages(&self, solution: &Solution) -> (f64, f64, f64, f64) {
        (
            solution.voltage(self.pins[0]).unwrap_or(0.0),
            solution.voltage(self.pins[1]).unwrap_or(0.0),
            solution.voltage(self.pins[2]).unwrap_or(0.0),
            solution.voltage(self.pins[3]).unwrap_or(0.0),
        )
    }

    pub(super) fn capacitances(&self, vgs: f64, vds: f64, vbs: f64) -> (f64, f64, f64) {
        capacitance_region(&self.params, vgs, vds, vbs)
    }

    fn stamp_caps(
        &self,
        ctx: &mut MnaContext,
        dt: Option<f64>,
        voltages: (f64, f64, f64, f64),
    ) -> Result<(), SimError> {
        let dt = match dt {
            Some(value) => value,
            None => return Ok(()),
        };
        let (vg, vd, vs, vb) = voltages;
        let sign = self.sign();
        let caps = self.capacitances(sign * (vg - vs), sign * (vd - vs), sign * (vb - vs));
        stamp_capacitor(ctx, self.pins[0], self.pins[2], caps.0, self.vgs_prev, dt)?;
        stamp_capacitor(ctx, self.pins[0], self.pins[1], caps.1, self.vgd_prev, dt)?;
        stamp_capacitor(ctx, self.pins[0], self.pins[3], caps.2, self.vgb_prev, dt)
    }
}

impl DeviceModel for MosfetModel {
    fn stamp(
        &mut self,
        ctx: &mut MnaContext,
        solution: &Solution,
        dt: Option<f64>,
    ) -> Result<(), SimError> {
        let voltages = self.voltages(solution);
        let (vg, vd, vs, vb) = voltages;
        let state = self.linearization(vg, vd, vs, vb);
        self.stamp_caps(ctx, dt, voltages)?;
        stamp_four_terminal(ctx, self.pins, state, voltages)
    }

    fn update(&mut self, solution: &Solution) -> Result<(), SimError> {
        let (vg, vd, vs, vb) = self.voltages(solution);
        self.vgs_prev = vg - vs;
        self.vgd_prev = vg - vd;
        self.vgb_prev = vg - vb;
        Ok(())
    }
}

pub(super) fn nmos_current_body(vgs: f64, vds: f64, vbs: f64, params: MosfetParams) -> f64 {
    let overdrive = vgs - threshold(params, vbs);
    if overdrive <= 0.0 {
        return 0.0;
    }
    let gain = params.kp * (params.width / params.length);
    let lambda = 1.0 + params.lambda * vds;
    if vds < overdrive {
        gain * (overdrive * vds - 0.5 * vds * vds) * lambda
    } else {
        0.5 * gain * overdrive * overdrive * lambda
    }
}

fn capacitance_region(params: &MosfetParams, vgs: f64, vds: f64, vbs: f64) -> (f64, f64, f64) {
    let overdrive = vgs - threshold(*params, vbs);
    if overdrive <= 0.0 {
        return (params.cgso, params.cgdo, params.cox);
    }
    if vds < overdrive {
        return (
            params.cgso + 0.5 * params.cox,
            params.cgdo + 0.5 * params.cox,
            0.0,
        );
    }
    (params.cgso + 2.0 / 3.0 * params.cox, params.cgdo, 0.0)
}

/// Returns small-signal MOSFET capacitances at the given bias.
pub fn mosfet_capacitances(
    kind: &DeviceKind,
    params: &BTreeMap<String, f64>,
    vg: f64,
    vd: f64,
    vs: f64,
    vb: f64,
) -> Result<(f64, f64, f64), SimError> {
    let params = MosfetParams::from_map(params)?;
    let sign = device_sign(kind);
    Ok(capacitance_region(
        &params,
        sign * (vg - vs),
        sign * (vd - vs),
        sign * (vb - vs),
    ))
}

fn device_sign(kind: &DeviceKind) -> f64 {
    if matches!(kind, DeviceKind::Pmos) {
        return -1.0;
    }
    1.0
}

pub(super) fn threshold(params: MosfetParams, vbs: f64) -> f64 {
    let bias = (params.phi - vbs).max(0.0);
    params.vto + params.gamma * (bias.sqrt() - params.phi.sqrt())
}
