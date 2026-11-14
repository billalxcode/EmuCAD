use super::{junction_critical_voltage, parameter, pnjlim, positive_parameter};
use std::collections::BTreeMap;

fn map(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_string(), *value))
        .collect()
}

#[test]
fn parameter_uses_value_when_present() {
    assert_eq!(parameter(&map(&[("x", 2.0)]), "x", 1.0), 2.0);
}

#[test]
fn parameter_uses_default_when_missing() {
    assert_eq!(parameter(&map(&[]), "x", 1.0), 1.0);
}

#[test]
fn positive_parameter_accepts_positive() {
    assert_eq!(
        positive_parameter(&map(&[("x", 2.0)]), "x", None).unwrap(),
        2.0
    );
}

#[test]
fn positive_parameter_rejects_zero() {
    assert!(positive_parameter(&map(&[("x", 0.0)]), "x", None).is_err());
}

#[test]
fn positive_parameter_rejects_negative() {
    assert!(positive_parameter(&map(&[("x", -1.0)]), "x", None).is_err());
}

#[test]
fn positive_parameter_rejects_missing() {
    assert!(positive_parameter(&map(&[]), "x", None).is_err());
}

#[test]
fn positive_parameter_rejects_nan() {
    assert!(positive_parameter(&map(&[("x", f64::NAN)]), "x", None).is_err());
}

#[test]
fn pnjlim_limits_large_forward_step() {
    let critical = junction_critical_voltage(1e-12, 0.02585);
    let limited = pnjlim(5.0, 0.0, 0.02585, critical);
    assert!(limited > 0.0 && limited < 0.2, "limited={limited}");
}

#[test]
fn pnjlim_keeps_small_step() {
    let critical = junction_critical_voltage(1e-12, 0.02585);
    let value = 0.62;
    assert_eq!(pnjlim(value, 0.60, 0.02585, critical), value);
}
