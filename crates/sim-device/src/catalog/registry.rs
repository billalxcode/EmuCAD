//! Registry that resolves catalog entries into solver devices.

use super::catalog;
use super::model::{ComponentSpec, ModelClass};
use crate::model::{DeviceModel, model_from_instance};
use sim_core::{CircuitBuilder, DeviceId, DeviceInstance, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

/// Request to instantiate a catalog entry into a concrete device.
#[derive(Clone, Debug)]
pub struct ComponentRequest {
    pub entry: &'static str,
    pub name: String,
    pub device_id: DeviceId,
    pub pins: Vec<NodeId>,
    pub parameters: BTreeMap<String, f64>,
}

impl ComponentRequest {
    /// Creates a request with default parameters.
    #[must_use]
    pub fn new(
        entry: &'static str,
        name: impl Into<String>,
        device_id: DeviceId,
        pins: Vec<NodeId>,
    ) -> Self {
        Self {
            entry,
            name: name.into(),
            device_id,
            pins,
            parameters: BTreeMap::new(),
        }
    }

    /// Adds or replaces a parameter override.
    #[must_use]
    pub fn with_parameter(mut self, name: impl Into<String>, value: f64) -> Self {
        self.parameters.insert(name.into(), value);
        self
    }
}

/// Registry that resolves catalog entries to solver models and circuits.
#[derive(Clone, Debug)]
pub struct ComponentRegistry {
    index: BTreeMap<&'static str, &'static ComponentSpec>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    /// Builds a registry from the static catalog.
    #[must_use]
    pub fn new() -> Self {
        let index = catalog().iter().map(|spec| (spec.id, spec)).collect();
        Self { index }
    }

    /// Returns all catalog entries.
    #[must_use]
    pub fn entries(&self) -> &'static [ComponentSpec] {
        catalog()
    }

    /// Finds one entry by ID.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&'static ComponentSpec> {
        self.index.get(id).copied()
    }

    /// Returns true when the entry has a solver-backed model.
    #[must_use]
    pub fn is_solver_backed(&self, id: &str) -> bool {
        self.find(id).is_some_and(|spec| spec.device.is_some())
    }

    /// Instantiates a solver-backed entry with default and override parameters.
    pub fn instantiate(&self, request: &ComponentRequest) -> Result<DeviceInstance, SimError> {
        let spec = self.require(request.entry)?;
        let device = self.require_device(spec)?;
        if request.pins.len() != usize::from(spec.pins) {
            return Err(SimError::InvalidNetlist(format!(
                "{} requires {} pins",
                spec.name, spec.pins
            )));
        }
        let parameters = parameters_for(spec, &request.parameters);
        Ok(DeviceInstance::new(
            request.device_id,
            request.name.clone(),
            device,
            request.pins.clone(),
            parameters,
        ))
    }

    /// Builds the solver model for one instantiated entry.
    pub fn model(
        &self,
        entry: &str,
        instance: &DeviceInstance,
    ) -> Result<Box<dyn DeviceModel>, SimError> {
        let spec = self.require(entry)?;
        self.require_device(spec)?;
        model_from_instance(instance)
    }

    /// Adds an entry directly to a circuit builder.
    pub fn add_to(
        &self,
        builder: &mut CircuitBuilder,
        request: &ComponentRequest,
    ) -> Result<DeviceId, SimError> {
        let spec = self.require(request.entry)?;
        let device = self.require_device(spec)?;
        let parameters = parameters_for(spec, &request.parameters);
        builder.device(
            request.name.clone(),
            device,
            request.pins.clone(),
            parameters,
        )
    }

    fn require(&self, id: &str) -> Result<&'static ComponentSpec, SimError> {
        self.find(id)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown component: {id}")))
    }

    fn require_device(&self, spec: &ComponentSpec) -> Result<DeviceKind, SimError> {
        spec.device
            .clone()
            .ok_or_else(|| SimError::UnsupportedDevice(spec.name.to_string()))
    }
}

fn parameters_for(
    spec: &ComponentSpec,
    overrides: &BTreeMap<String, f64>,
) -> BTreeMap<String, f64> {
    let mut parameters = BTreeMap::new();
    parameters.insert(spec.parameter.to_string(), spec.default);
    if spec.class == ModelClass::LogicGate {
        parameters.insert("gate".to_string(), f64::from(spec.variant));
    }
    parameters.extend(overrides.iter().map(|(name, value)| (name.clone(), *value)));
    parameters
}
