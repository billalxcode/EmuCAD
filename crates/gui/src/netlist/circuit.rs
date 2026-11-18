//! Netlist-to-circuit conversion and DC solving.

use super::Netlist;
use analysis::{DcConfig, DcResult, dc_operating_point};
use device::{ComponentRegistry, ComponentRequest};
use kernel::{Circuit, CircuitBuilder, DeviceId, NodeId};

/// Builds a solver circuit from an extracted netlist.
pub(crate) fn build_circuit(netlist: &Netlist) -> Result<Circuit, String> {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let nodes = add_nodes(&mut builder, netlist)?;
    for component in &netlist.components {
        add_component(&mut builder, &registry, component, &nodes)?;
    }
    builder.build().map_err(|error| error.to_string())
}

/// Solves the DC operating point of an extracted netlist.
pub(crate) fn solve_dc(netlist: &Netlist) -> Result<DcResult, String> {
    let circuit = build_circuit(netlist)?;
    dc_operating_point(&circuit, &DcConfig::default()).map_err(|error| error.to_string())
}

/// Returns node voltages in netlist order.
pub(crate) fn node_voltages(netlist: &Netlist, result: &DcResult) -> Vec<(String, f64)> {
    netlist
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let raw = u32::try_from(index).expect("node index fits u32");
            let voltage = result
                .solution
                .voltage(NodeId::new(raw))
                .unwrap_or(f64::NAN);
            (node.name.clone(), voltage)
        })
        .collect()
}

fn add_nodes(builder: &mut CircuitBuilder, netlist: &Netlist) -> Result<Vec<NodeId>, String> {
    if netlist.ground != 0 {
        return Err("ground node must be zero".to_string());
    }
    let mut nodes = vec![builder.ground(); netlist.nodes.len()];
    for (index, node) in netlist.nodes.iter().enumerate().skip(1) {
        nodes[index] = builder
            .node(node.name.clone())
            .map_err(|error| error.to_string())?;
    }
    Ok(nodes)
}

fn add_component(
    builder: &mut CircuitBuilder,
    registry: &ComponentRegistry,
    component: &super::NetlistComponent,
    nodes: &[NodeId],
) -> Result<(), String> {
    let pins = component.pins.iter().map(|&index| nodes[index]).collect();
    let mut request = ComponentRequest::new(
        component.part,
        component.label.clone(),
        DeviceId::new(0),
        pins,
    );
    request.parameters = component.parameters.clone();
    registry
        .add_to(builder, &request)
        .map_err(|error| error.to_string())?;
    Ok(())
}
