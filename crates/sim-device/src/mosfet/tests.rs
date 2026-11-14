use super::{MosfetModel, MosfetParams, mosfet_capacitances, nmos_current_body, threshold};
use sim_core::{DeviceKind, NodeId};
use std::collections::BTreeMap;

const VTO: f64 = 0.5;

fn params() -> MosfetParams {
    MosfetParams::new(VTO, 1e-3, 0.0, 1e-6, 1e-6)
}

fn body_params() -> MosfetParams {
    MosfetParams {
        vto: VTO,
        kp: 1e-3,
        lambda: 0.0,
        width: 1e-6,
        length: 1e-6,
        gamma: 0.5,
        phi: 0.7,
        cox: 0.0,
        cgso: 0.0,
        cgdo: 0.0,
    }
}

fn cap_params() -> MosfetParams {
    MosfetParams {
        vto: VTO,
        kp: 1e-3,
        lambda: 0.0,
        width: 1e-6,
        length: 1e-6,
        gamma: 0.0,
        phi: 0.7,
        cox: 1e-12,
        cgso: 1e-13,
        cgdo: 2e-13,
    }
}

fn model(kind: DeviceKind, params: MosfetParams) -> MosfetModel {
    MosfetModel {
        kind,
        params,
        pins: [
            NodeId::new(1),
            NodeId::new(2),
            NodeId::GROUND,
            NodeId::new(3),
        ],
        vgs_prev: 0.0,
        vgd_prev: 0.0,
        vgb_prev: 0.0,
    }
}

#[test]
fn cutoff_current_is_zero() {
    assert_eq!(nmos_current_body(0.4, 1.0, 0.0, params()), 0.0);
    assert_eq!(nmos_current_body(0.0, 1.0, 0.0, params()), 0.0);
}

#[test]
fn linear_region_matches_formula() {
    let expected = 1e-3 * (0.5 * 0.2 - 0.5 * 0.2 * 0.2);
    let actual = nmos_current_body(1.0, 0.2, 0.0, params());
    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn saturation_region_matches_formula() {
    let expected = 0.5 * 1e-3 * 0.5 * 0.5;
    let actual = nmos_current_body(1.0, 1.0, 0.0, params());
    assert!((actual - expected).abs() < 1e-12);
}

#[test]
fn lambda_raises_saturation_current() {
    let base = nmos_current_body(1.0, 1.0, 0.0, params());
    let modulated = nmos_current_body(1.0, 1.0, 0.0, MosfetParams::new(VTO, 1e-3, 0.1, 1e-6, 1e-6));
    assert!(modulated > base);
}

#[test]
fn width_over_length_scales_current() {
    let base = nmos_current_body(1.0, 1.0, 0.0, params());
    let wider = nmos_current_body(1.0, 1.0, 0.0, MosfetParams::new(VTO, 1e-3, 0.0, 2e-6, 1e-6));
    assert!((wider - 2.0 * base).abs() < 1e-12);
}

#[test]
fn numeric_derivative_matches_analytic() {
    let params = params();
    let step = 1e-6;
    let base = nmos_current_body(1.0, 0.2, 0.0, params);
    let gm = (nmos_current_body(1.0 + step, 0.2, 0.0, params) - base) / step;
    let gds = (nmos_current_body(1.0, 0.2 + step, 0.0, params) - base) / step;
    assert!((gm - 1e-3 * 0.2).abs() < 1e-6);
    assert!((gds - 1e-3 * 0.3).abs() < 1e-6);
}

#[test]
fn body_bias_raises_threshold_and_lowers_current() {
    let base = nmos_current_body(1.0, 1.0, 0.0, body_params());
    let biased = nmos_current_body(1.0, 1.0, -1.0, body_params());
    assert!(biased < base);
    assert!(threshold(body_params(), -1.0) > threshold(body_params(), 0.0));
}

#[test]
fn gmb_is_positive_for_reverse_body_bias() {
    let model = model(DeviceKind::Nmos, body_params());
    let gmb = model.linearization(1.2, 1.0, 0.0, -1.0).gmb;
    assert!(gmb > 0.0, "gmb={gmb}");
}

#[test]
fn public_capacitances_follow_bias_and_kind() {
    let mut map = BTreeMap::new();
    map.insert("vto".to_string(), VTO);
    map.insert("cox".to_string(), 1e-12);
    map.insert("cgso".to_string(), 1e-13);
    map.insert("cgdo".to_string(), 2e-13);
    let nmos = mosfet_capacitances(&DeviceKind::Nmos, &map, 1.0, 1.0, 0.0, 0.0).expect("nmos");
    let pmos = mosfet_capacitances(&DeviceKind::Pmos, &map, 0.0, 0.0, 1.0, 1.0).expect("pmos");
    assert!((nmos.0 - (1e-13 + 2.0 / 3.0 * 1e-12)).abs() < 1e-20);
    assert_eq!(nmos, pmos);
}

#[test]
fn capacitance_partition_matches_region() {
    let model = model(DeviceKind::Nmos, cap_params());
    let cutoff = model.capacitances(0.4, 1.0, 0.0);
    assert!((cutoff.0 - 1e-13).abs() < 1e-20);
    assert!((cutoff.2 - 1e-12).abs() < 1e-20);
    let linear = model.capacitances(1.0, 0.2, 0.0);
    assert!((linear.0 - 6e-13).abs() < 1e-20);
    assert!((linear.1 - 7e-13).abs() < 1e-20);
    let saturation = model.capacitances(1.0, 1.0, 0.0);
    assert!((saturation.0 - (1e-13 + 2.0 / 3.0 * 1e-12)).abs() < 1e-20);
    assert!((saturation.1 - 2e-13).abs() < 1e-20);
}
