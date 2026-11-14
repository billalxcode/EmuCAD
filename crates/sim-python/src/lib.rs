//! Python bindings for the EmuCAD component catalog and netlist builder.

#[path = "catalog.rs"]
mod component_catalog;
mod netlist;
mod python;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Returns all catalog entries as dictionaries.
#[pyfunction]
fn catalog(py: Python<'_>) -> PyResult<Py<PyList>> {
    let entries = component_catalog::entries();
    let list = PyList::empty(py);
    for entry in &entries {
        list.append(python::entry_dict(py, entry)?)?;
    }
    Ok(list.unbind())
}

/// Finds one catalog entry by ID.
#[pyfunction]
fn find_component(py: Python<'_>, id: &str) -> PyResult<Option<Py<PyDict>>> {
    let Some(entry) = component_catalog::find(id) else {
        return Ok(None);
    };
    Ok(Some(python::entry_dict(py, &entry)?.unbind()))
}

/// Builds a topology-validated netlist from a request dictionary.
#[pyfunction]
fn build_netlist(py: Python<'_>, request: python::PyNetlistRequest) -> PyResult<Py<PyDict>> {
    let request = netlist::NetlistRequest::try_from(request).map_err(PyValueError::new_err)?;
    let snapshot = netlist::build(&request)
        .map_err(|error| PyValueError::new_err(error.message().to_string()))?;
    Ok(python::snapshot_dict(py, &snapshot)?.unbind())
}

#[pymodule]
fn emucad(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(catalog, module)?)?;
    module.add_function(wrap_pyfunction!(find_component, module)?)?;
    module.add_function(wrap_pyfunction!(build_netlist, module)?)?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
