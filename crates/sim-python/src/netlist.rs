//! Pure netlist construction from Python-friendly requests.

use sim_core::{
    Circuit, CircuitBuilder, Device, DeviceId, DeviceInstance, Node, NodeId,
    NodeKind as CoreNodeKind,
};
use sim_device::{ComponentRegistry, ComponentRequest};
use std::collections::BTreeMap;
use std::fmt;

/// Requested node kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NodeKind {
    #[default]
    Signal,
    Supply,
}

/// One requested node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeInput {
    pub name: String,
    pub kind: NodeKind,
}

/// One requested component.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ComponentInput {
    pub name: String,
    pub part: String,
    pub pins: Vec<String>,
    pub parameters: BTreeMap<String, f64>,
}

/// Full netlist request from Python.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct NetlistRequest {
    pub nodes: Vec<NodeInput>,
    pub components: Vec<ComponentInput>,
}

/// Node in a built netlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeRecord {
    pub id: u32,
    pub name: String,
    pub kind: &'static str,
}

/// Device in a built netlist.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DeviceRecord {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub pins: Vec<String>,
    pub parameters: BTreeMap<String, f64>,
}

/// Built netlist snapshot.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetlistSnapshot {
    pub nodes: Vec<NodeRecord>,
    pub devices: Vec<DeviceRecord>,
}

/// Netlist construction error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuildError {
    message: String,
}

impl BuildError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn from_sim(error: sim_core::SimError) -> Self {
        Self::new(error.to_string())
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BuildError {}

/// Builds a topology-validated circuit from a request.
pub(crate) fn build(request: &NetlistRequest) -> Result<NetlistSnapshot, BuildError> {
    let mut builder = CircuitBuilder::new();
    let nodes = add_nodes(&mut builder, &request.nodes)?;
    let registry = ComponentRegistry::new();
    add_components(&mut builder, &registry, &nodes, &request.components)?;
    let circuit = builder.build_topology().map_err(BuildError::from_sim)?;
    Ok(snapshot(&circuit))
}

fn add_nodes(
    builder: &mut CircuitBuilder,
    requests: &[NodeInput],
) -> Result<BTreeMap<String, NodeId>, BuildError> {
    let mut nodes = BTreeMap::new();
    nodes.insert("0".to_string(), NodeId::GROUND);
    for request in requests {
        add_node(builder, &mut nodes, request)?;
    }
    Ok(nodes)
}

fn add_node(
    builder: &mut CircuitBuilder,
    nodes: &mut BTreeMap<String, NodeId>,
    request: &NodeInput,
) -> Result<(), BuildError> {
    if request.name == "0" {
        return Err(BuildError::new("node 0 is reserved for ground"));
    }
    if nodes.contains_key(&request.name) {
        return Err(BuildError::new(format!("duplicate node: {}", request.name)));
    }
    let id = create_node(builder, request).map_err(BuildError::from_sim)?;
    nodes.insert(request.name.clone(), id);
    Ok(())
}

fn create_node(
    builder: &mut CircuitBuilder,
    request: &NodeInput,
) -> Result<NodeId, sim_core::SimError> {
    match request.kind {
        NodeKind::Signal => builder.node(request.name.clone()),
        NodeKind::Supply => builder.supply(request.name.clone()),
    }
}

fn add_components(
    builder: &mut CircuitBuilder,
    registry: &ComponentRegistry,
    nodes: &BTreeMap<String, NodeId>,
    requests: &[ComponentInput],
) -> Result<(), BuildError> {
    for request in requests {
        add_component(builder, registry, nodes, request)?;
    }
    Ok(())
}

fn add_component(
    builder: &mut CircuitBuilder,
    registry: &ComponentRegistry,
    nodes: &BTreeMap<String, NodeId>,
    request: &ComponentInput,
) -> Result<(), BuildError> {
    let spec = registry
        .find(&request.part)
        .ok_or_else(|| BuildError::new(format!("unknown component: {}", request.part)))?;
    let pins = resolve_pins(nodes, &request.pins)?;
    validate_parameters(&request.parameters)?;
    let mut component = ComponentRequest::new(spec.id, &request.name, DeviceId::new(0), pins);
    for (name, value) in &request.parameters {
        component = component.with_parameter(name.clone(), *value);
    }
    registry
        .add_to(builder, &component)
        .map_err(BuildError::from_sim)?;
    Ok(())
}

fn validate_parameters(parameters: &BTreeMap<String, f64>) -> Result<(), BuildError> {
    for (name, value) in parameters {
        if !value.is_finite() {
            return Err(BuildError::new(format!("parameter {name} must be finite")));
        }
    }
    Ok(())
}

fn resolve_pins(
    nodes: &BTreeMap<String, NodeId>,
    pins: &[String],
) -> Result<Vec<NodeId>, BuildError> {
    pins.iter().map(|pin| resolve_pin(nodes, pin)).collect()
}

fn resolve_pin(nodes: &BTreeMap<String, NodeId>, name: &str) -> Result<NodeId, BuildError> {
    nodes
        .get(name)
        .copied()
        .ok_or_else(|| BuildError::new(format!("unknown node: {name}")))
}

fn snapshot(circuit: &Circuit) -> NetlistSnapshot {
    NetlistSnapshot {
        nodes: circuit.nodes().iter().map(node_record).collect(),
        devices: circuit
            .devices()
            .iter()
            .map(|device| device_record(circuit, device))
            .collect(),
    }
}

fn node_record(node: &Node) -> NodeRecord {
    NodeRecord {
        id: node.id().raw(),
        name: node.name().to_string(),
        kind: node_kind(node.kind()),
    }
}

const fn node_kind(kind: CoreNodeKind) -> &'static str {
    match kind {
        CoreNodeKind::Ground => "ground",
        CoreNodeKind::Signal => "signal",
        CoreNodeKind::Supply => "supply",
    }
}

fn device_record(circuit: &Circuit, device: &DeviceInstance) -> DeviceRecord {
    DeviceRecord {
        id: device.id().raw(),
        name: device.name().to_string(),
        kind: device.kind().name().to_string(),
        pins: device
            .pins()
            .iter()
            .map(|id| pin_name(circuit, *id))
            .collect(),
        parameters: device.parameters().clone(),
    }
}

fn pin_name(circuit: &Circuit, id: NodeId) -> String {
    circuit
        .node(id)
        .map_or_else(|| id.raw().to_string(), |node| node.name().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESISTOR: &str = "passive.resistor.ideal_resistor";

    fn resistor(name: &str, first: &str, second: &str, value: Option<f64>) -> ComponentInput {
        let mut parameters = BTreeMap::new();
        if let Some(value) = value {
            parameters.insert("resistance".to_string(), value);
        }
        ComponentInput {
            name: name.to_string(),
            part: RESISTOR.to_string(),
            pins: vec![first.to_string(), second.to_string()],
            parameters,
        }
    }

    fn divider() -> NetlistRequest {
        NetlistRequest {
            nodes: vec![
                NodeInput {
                    name: "vcc".to_string(),
                    kind: NodeKind::Supply,
                },
                NodeInput {
                    name: "out".to_string(),
                    kind: NodeKind::Signal,
                },
            ],
            components: vec![
                resistor("r1", "vcc", "out", Some(1000.0)),
                resistor("r2", "out", "0", Some(2000.0)),
            ],
        }
    }

    #[test]
    fn builds_voltage_divider() {
        let snapshot = build(&divider()).expect("build");
        assert_eq!(snapshot.nodes.len(), 3);
        assert_eq!(snapshot.devices.len(), 2);
        assert_eq!(snapshot.nodes[0].name, "0");
        assert_eq!(snapshot.nodes[0].kind, "ground");
        assert_eq!(snapshot.nodes[1].kind, "supply");
        assert_eq!(snapshot.devices[0].pins, ["vcc", "out"]);
        assert_eq!(snapshot.devices[1].pins, ["out", "0"]);
    }

    #[test]
    fn uses_catalog_default_parameters() {
        let mut request = divider();
        request.components[0] = resistor("r1", "vcc", "out", None);
        let snapshot = build(&request).expect("build");
        assert_eq!(snapshot.devices[0].parameters["resistance"], 1000.0);
    }

    #[test]
    fn rejects_unknown_component() {
        let mut request = divider();
        request.components[0].part = "missing.part".to_string();
        assert!(
            build(&request)
                .expect_err("error")
                .message()
                .contains("unknown component")
        );
    }

    #[test]
    fn rejects_unknown_node() {
        let mut request = divider();
        request.components[0].pins[1] = "missing".to_string();
        assert!(
            build(&request)
                .expect_err("error")
                .message()
                .contains("unknown node")
        );
    }

    #[test]
    fn rejects_duplicate_node() {
        let mut request = divider();
        request.nodes.push(request.nodes[0].clone());
        assert!(
            build(&request)
                .expect_err("error")
                .message()
                .contains("duplicate node")
        );
    }

    #[test]
    fn rejects_wrong_pin_count() {
        let mut request = divider();
        request.components[0].pins.pop();
        assert!(
            build(&request)
                .expect_err("error")
                .message()
                .contains("requires 2 pins")
        );
    }

    #[test]
    fn rejects_non_finite_parameters() {
        let mut request = divider();
        request.components[0]
            .parameters
            .insert("resistance".to_string(), f64::NAN);
        assert!(
            build(&request)
                .expect_err("error")
                .message()
                .contains("must be finite")
        );
    }

    #[test]
    fn rejects_unused_nodes() {
        let mut request = divider();
        request.nodes.push(NodeInput {
            name: "spare".to_string(),
            kind: NodeKind::Signal,
        });
        assert!(build(&request).is_err());
    }
}
