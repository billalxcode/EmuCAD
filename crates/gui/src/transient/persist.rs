//! Save and load transient traces as JSON metadata plus binary samples.

use super::{TransientDevice, TransientTrace};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"EMTR";
const VERSION: u32 = 1;
const MAX_ROWS: u64 = 1_000_000;
const MAX_NODES: u64 = 10_000;
const MAX_DEVICES: u64 = 10_000;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceFile {
    format: String,
    version: u32,
    dt: f64,
    adaptive: bool,
    nodes: Vec<String>,
    devices: Vec<DeviceFile>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceFile {
    label: String,
    part: String,
}

/// Saves a transient trace next to a project path.
pub(crate) fn save(base: &Path, trace: &TransientTrace) -> Result<(), String> {
    validate_trace(trace)?;
    let bin = sidecar(base, "transient.bin");
    let json = sidecar(base, "transient.json");
    write_binary(&bin, trace)?;
    if let Err(error) = write_json(&json, trace) {
        let _ = fs::remove_file(&bin);
        return Err(error);
    }
    Ok(())
}

/// Removes transient sidecar files next to a project path.
pub(crate) fn remove(base: &Path) {
    let _ = fs::remove_file(sidecar(base, "transient.bin"));
    let _ = fs::remove_file(sidecar(base, "transient.json"));
}

/// Loads a transient trace next to a project path when both files exist.
pub(crate) fn load(base: &Path) -> Result<Option<TransientTrace>, String> {
    let bin = sidecar(base, "transient.bin");
    let json = sidecar(base, "transient.json");
    if !bin.exists() || !json.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&json).map_err(|error| format!("transient json: {error}"))?;
    let meta: TraceFile =
        serde_json::from_str(&text).map_err(|error| format!("transient json: {error}"))?;
    validate_meta(&meta)?;
    let trace = read_binary(&bin, &meta)?;
    Ok(Some(trace))
}

fn sidecar(base: &Path, suffix: &str) -> PathBuf {
    let mut name = base.as_os_str().to_os_string();
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

fn validate_trace(trace: &TransientTrace) -> Result<(), String> {
    if !trace.dt.is_finite() || trace.dt <= 0.0 {
        return Err("transient dt is invalid".to_string());
    }
    if trace.times.len() != trace.values.len() || trace.times.len() != trace.currents.len() {
        return Err("transient row counts differ".to_string());
    }
    if trace
        .values
        .iter()
        .any(|row| row.len() != trace.nodes.len())
    {
        return Err("transient node counts differ".to_string());
    }
    if trace
        .currents
        .iter()
        .any(|row| row.len() != trace.devices.len())
    {
        return Err("transient device counts differ".to_string());
    }
    Ok(())
}

fn validate_meta(meta: &TraceFile) -> Result<(), String> {
    if meta.format != "emucad-transient" || meta.version != VERSION {
        return Err("unsupported transient file".to_string());
    }
    if !meta.dt.is_finite() || meta.dt <= 0.0 {
        return Err("transient dt is invalid".to_string());
    }
    Ok(())
}

fn write_json(path: &Path, trace: &TransientTrace) -> Result<(), String> {
    let file = TraceFile {
        format: "emucad-transient".to_string(),
        version: VERSION,
        dt: trace.dt,
        adaptive: trace.adaptive,
        nodes: trace.nodes.clone(),
        devices: trace
            .devices
            .iter()
            .map(|device| DeviceFile {
                label: device.label.clone(),
                part: device.part.clone(),
            })
            .collect(),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| format!("transient json: {error}"))
}

fn write_binary(path: &Path, trace: &TransientTrace) -> Result<(), String> {
    let mut file = fs::File::create(path).map_err(|error| format!("transient bin: {error}"))?;
    file.write_all(MAGIC).map_err(io_error)?;
    write_u32(&mut file, VERSION)?;
    write_f64(&mut file, trace.dt)?;
    file.write_all(&[u8::from(trace.adaptive)])
        .map_err(io_error)?;
    write_u64(&mut file, trace.nodes.len() as u64)?;
    write_u64(&mut file, trace.times.len() as u64)?;
    write_u64(&mut file, trace.devices.len() as u64)?;
    for index in 0..trace.times.len() {
        write_f64(&mut file, trace.times[index])?;
        for value in &trace.values[index] {
            write_f64(&mut file, *value)?;
        }
        write_currents(&mut file, &trace.currents[index])?;
    }
    Ok(())
}

fn write_currents(file: &mut fs::File, currents: &[Option<f64>]) -> Result<(), String> {
    write_u64(file, currents.len() as u64)?;
    for current in currents {
        match current {
            Some(value) => {
                file.write_all(&[1]).map_err(io_error)?;
                write_f64(file, *value)?;
            }
            None => {
                file.write_all(&[0]).map_err(io_error)?;
            }
        }
    }
    Ok(())
}

fn write_u32(file: &mut fs::File, value: u32) -> Result<(), String> {
    file.write_all(&value.to_le_bytes()).map_err(io_error)
}

fn write_u64(file: &mut fs::File, value: u64) -> Result<(), String> {
    file.write_all(&value.to_le_bytes()).map_err(io_error)
}

fn write_f64(file: &mut fs::File, value: f64) -> Result<(), String> {
    file.write_all(&value.to_le_bytes()).map_err(io_error)
}

fn read_binary(path: &Path, meta: &TraceFile) -> Result<TransientTrace, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("transient bin: {error}"))?;
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).map_err(io_error)?;
    if &magic != MAGIC {
        return Err("transient bin magic mismatch".to_string());
    }
    let version = read_u32(&mut file)?;
    if version != VERSION {
        return Err("transient bin version mismatch".to_string());
    }
    let dt = read_f64(&mut file)?;
    let adaptive = read_bool(&mut file)?;
    let node_count = read_count(&mut file, MAX_NODES)?;
    let sample_count = read_count(&mut file, MAX_ROWS)?;
    let device_count = read_count(&mut file, MAX_DEVICES)?;
    if node_count != meta.nodes.len() || device_count != meta.devices.len() {
        return Err("transient metadata counts differ".to_string());
    }
    let mut trace = TransientTrace::empty(super::TraceMetadata {
        nodes: meta.nodes.clone(),
        devices: meta
            .devices
            .iter()
            .map(|device| TransientDevice {
                label: device.label.clone(),
                part: device.part.clone(),
            })
            .collect(),
        dt,
        adaptive,
    });
    for _ in 0..sample_count {
        read_sample(&mut file, &mut trace, node_count, device_count)?;
    }
    Ok(trace)
}

fn read_sample(
    file: &mut fs::File,
    trace: &mut TransientTrace,
    node_count: usize,
    device_count: usize,
) -> Result<(), String> {
    trace.times.push(read_f64(file)?);
    let mut values = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        values.push(read_f64(file)?);
    }
    trace.values.push(values);
    trace.currents.push(read_currents(file, device_count)?);
    Ok(())
}

fn read_currents(file: &mut fs::File, count: usize) -> Result<Vec<Option<f64>>, String> {
    let stored = read_count(file, MAX_DEVICES)?;
    if stored != count {
        return Err("transient current count mismatch".to_string());
    }
    let mut currents = Vec::with_capacity(count);
    for _ in 0..count {
        if read_bool(file)? {
            currents.push(Some(read_f64(file)?));
        } else {
            currents.push(None);
        }
    }
    Ok(currents)
}

fn read_u32(file: &mut fs::File) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    file.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(file: &mut fs::File) -> Result<u64, String> {
    let mut bytes = [0_u8; 8];
    file.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_f64(file: &mut fs::File) -> Result<f64, String> {
    let mut bytes = [0_u8; 8];
    file.read_exact(&mut bytes).map_err(io_error)?;
    Ok(f64::from_le_bytes(bytes))
}

fn read_bool(file: &mut fs::File) -> Result<bool, String> {
    let mut byte = [0_u8; 1];
    file.read_exact(&mut byte).map_err(io_error)?;
    Ok(byte[0] != 0)
}

fn read_count(file: &mut fs::File, limit: u64) -> Result<usize, String> {
    let value = read_u64(file)?;
    if value > limit {
        return Err("transient file is too large".to_string());
    }
    usize::try_from(value).map_err(|_| "transient count overflow".to_string())
}

fn io_error(error: std::io::Error) -> String {
    format!("transient io: {error}")
}

#[cfg(test)]
mod tests {
    use super::{load, save};
    use crate::transient::{TraceMetadata, TransientDevice, TransientTrace};
    use std::path::PathBuf;

    fn temp_base(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("emucad-transient-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        dir.join("project.emu.json")
    }

    fn trace() -> TransientTrace {
        let mut trace = TransientTrace::empty(TraceMetadata {
            nodes: vec!["0".to_string(), "out".to_string()],
            devices: vec![TransientDevice {
                label: "R1".to_string(),
                part: "passive.resistor.ideal_resistor".to_string(),
            }],
            dt: 1e-6,
            adaptive: false,
        });
        trace.times = vec![1e-6, 2e-6];
        trace.values = vec![vec![0.0, 2.5], vec![0.0, 5.0]];
        trace.currents = vec![vec![Some(0.0025)], vec![Some(0.005)]];
        trace
    }

    #[test]
    fn transient_round_trips() {
        let base = temp_base("roundtrip");
        let expected = trace();
        save(&base, &expected).expect("save");
        let loaded = load(&base).expect("load").expect("trace");
        assert_eq!(loaded, expected);
    }

    #[test]
    fn missing_sidecars_return_none() {
        let base = temp_base("missing");
        assert!(load(&base).expect("load").is_none());
    }

    #[test]
    fn corrupt_binary_is_rejected() {
        let base = temp_base("corrupt");
        save(&base, &trace()).expect("save");
        let bin = base.with_file_name("project.emu.json.transient.bin");
        std::fs::write(bin, b"bad").expect("write");
        assert!(load(&base).is_err());
    }
}
