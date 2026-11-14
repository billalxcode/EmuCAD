use super::*;

pub(crate) fn params(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}

pub(super) fn pulse_parameters(
    levels: (f64, f64),
    timing: (f64, f64, f64, f64, f64),
) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("v1".to_string(), levels.0);
    map.insert("v2".to_string(), levels.1);
    map.insert("delay".to_string(), timing.0);
    map.insert("rise".to_string(), timing.1);
    map.insert("fall".to_string(), timing.2);
    map.insert("width".to_string(), timing.3);
    map.insert("period".to_string(), timing.4);
    map
}

pub(crate) fn mos_params(width: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert("vto".to_string(), 0.7);
    map.insert("kp".to_string(), 1e-3);
    map.insert("lambda".to_string(), 0.02);
    map.insert("w".to_string(), width);
    map.insert("l".to_string(), 1e-6);
    map
}
