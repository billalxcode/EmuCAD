//! PyO3 conversion layer.

use crate::netlist::{
    ComponentInput, DeviceRecord, NetlistRequest, NetlistSnapshot, NodeInput, NodeKind, NodeRecord,
};
use device::CatalogRecord;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::BTreeMap;

/// Python node input.
#[derive(Debug, FromPyObject)]
#[pyo3(from_item_all)]
pub(crate) struct PyNodeInput {
    pub name: String,
    #[pyo3(default)]
    pub kind: Option<String>,
}

/// Python component input.
#[derive(Debug, FromPyObject)]
#[pyo3(from_item_all)]
pub(crate) struct PyComponentInput {
    pub name: String,
    pub part: String,
    pub pins: Vec<String>,
    #[pyo3(default)]
    pub parameters: Option<BTreeMap<String, f64>>,
}

/// Python netlist request.
#[derive(Debug, FromPyObject)]
#[pyo3(from_item_all)]
pub(crate) struct PyNetlistRequest {
    #[pyo3(default)]
    pub nodes: Option<Vec<PyNodeInput>>,
    pub components: Vec<PyComponentInput>,
}

impl TryFrom<PyNodeInput> for NodeInput {
    type Error = String;

    fn try_from(input: PyNodeInput) -> Result<Self, Self::Error> {
        Ok(Self {
            name: input.name,
            kind: node_kind(input.kind.as_deref())?,
        })
    }
}

impl From<PyComponentInput> for ComponentInput {
    fn from(input: PyComponentInput) -> Self {
        Self {
            name: input.name,
            part: input.part,
            pins: input.pins,
            parameters: input.parameters.unwrap_or_default(),
        }
    }
}

impl TryFrom<PyNetlistRequest> for NetlistRequest {
    type Error = String;

    fn try_from(input: PyNetlistRequest) -> Result<Self, Self::Error> {
        let nodes = input
            .nodes
            .unwrap_or_default()
            .into_iter()
            .map(NodeInput::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let components = input.components.into_iter().map(Into::into).collect();
        Ok(Self { nodes, components })
    }
}

fn node_kind(kind: Option<&str>) -> Result<NodeKind, String> {
    match kind {
        None | Some("signal") => Ok(NodeKind::Signal),
        Some("supply") => Ok(NodeKind::Supply),
        Some(other) => Err(format!("unknown node kind: {other}")),
    }
}

/// Converts one catalog entry into a Python dictionary.
pub(crate) fn entry_dict<'py>(
    py: Python<'py>,
    entry: &CatalogRecord,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("id", entry.id)?;
    dict.set_item("name", entry.name)?;
    dict.set_item("family", entry.family)?;
    dict.set_item("model_class", &entry.model_class)?;
    dict.set_item("device", entry.device.as_deref())?;
    dict.set_item("pins", entry.pins)?;
    dict.set_item("parameter", entry.parameter)?;
    dict.set_item("default", entry.default)?;
    dict.set_item("unit", entry.unit)?;
    dict.set_item("variant", entry.variant)?;
    dict.set_item("solver_backed", entry.solver_backed)?;
    Ok(dict)
}

/// Converts a netlist snapshot into a Python dictionary.
pub(crate) fn snapshot_dict<'py>(
    py: Python<'py>,
    snapshot: &NetlistSnapshot,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("nodes", node_list(py, &snapshot.nodes)?)?;
    dict.set_item("devices", device_list(py, &snapshot.devices)?)?;
    Ok(dict)
}

fn node_list<'py>(py: Python<'py>, nodes: &[NodeRecord]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for node in nodes {
        list.append(node_dict(py, node)?)?;
    }
    Ok(list)
}

fn device_list<'py>(py: Python<'py>, devices: &[DeviceRecord]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for device in devices {
        list.append(device_dict(py, device)?)?;
    }
    Ok(list)
}

fn node_dict<'py>(py: Python<'py>, node: &NodeRecord) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("id", node.id)?;
    dict.set_item("name", &node.name)?;
    dict.set_item("kind", node.kind)?;
    Ok(dict)
}

fn device_dict<'py>(py: Python<'py>, device: &DeviceRecord) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("id", device.id)?;
    dict.set_item("name", &device.name)?;
    dict.set_item("kind", &device.kind)?;
    dict.set_item("pins", &device.pins)?;
    dict.set_item("parameters", parameter_dict(py, &device.parameters)?)?;
    Ok(dict)
}

fn parameter_dict<'py>(
    py: Python<'py>,
    parameters: &BTreeMap<String, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (name, value) in parameters {
        dict.set_item(name, *value)?;
    }
    Ok(dict)
}
