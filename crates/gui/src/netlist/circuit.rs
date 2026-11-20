//! Netlist-to-circuit conversion and DC solving.

use super::{McuNetlist, Netlist};
use analysis::{DcConfig, DcResult, dc_operating_point, dc_terminal_currents};
use device::{ComponentRegistry, ComponentRequest};
use kernel::{Circuit, CircuitBuilder, DeviceId, DeviceKind, NodeId};
use std::collections::BTreeMap;

/// Builds a solver circuit from an extracted netlist.
pub(crate) fn build_circuit(netlist: &Netlist) -> Result<Circuit, String> {
    build_circuit_with_sources(netlist, &[]).map(|(circuit, _)| circuit)
}

/// Builds a circuit with hidden sources for selected MCU package pins.
pub(crate) fn build_circuit_with_sources(
    netlist: &Netlist,
    source_pins: &[u8],
) -> Result<(Circuit, BTreeMap<u8, DeviceId>), String> {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let nodes = add_nodes(&mut builder, netlist)?;
    for component in &netlist.components {
        add_component(&mut builder, &registry, component, &nodes)?;
    }
    let sources = add_mcu_sources(&mut builder, netlist, source_pins, &nodes)?;
    let circuit = builder.build().map_err(|error| error.to_string())?;
    Ok((circuit, sources))
}

fn add_mcu_sources(
    builder: &mut CircuitBuilder,
    netlist: &Netlist,
    source_pins: &[u8],
    nodes: &[NodeId],
) -> Result<BTreeMap<u8, DeviceId>, String> {
    let Some(mcu) = netlist_mcu(netlist, source_pins)? else {
        return Ok(BTreeMap::new());
    };
    let ground = builder.ground();
    let mut sources = BTreeMap::new();
    for &pin in source_pins {
        let node = mcu_source_node(mcu, netlist.ground, pin)?;
        let id = add_voltage_source(builder, pin, nodes[node], ground)?;
        sources.insert(pin, id);
    }
    Ok(sources)
}

fn netlist_mcu<'a>(
    netlist: &'a Netlist,
    source_pins: &[u8],
) -> Result<Option<&'a McuNetlist>, String> {
    match netlist.mcu.as_ref() {
        Some(mcu) => Ok(Some(mcu)),
        None if source_pins.is_empty() => Ok(None),
        None => Err("netlist has no MCU".to_string()),
    }
}

fn mcu_source_node(mcu: &McuNetlist, ground: usize, pin: u8) -> Result<usize, String> {
    let node = *mcu
        .pins
        .get(&pin)
        .ok_or_else(|| format!("MCU pin {pin} is not mapped"))?;
    if node == ground {
        return Err(format!("MCU pin {pin} is connected to ground"));
    }
    Ok(node)
}

fn add_voltage_source(
    builder: &mut CircuitBuilder,
    pin: u8,
    node: NodeId,
    ground: NodeId,
) -> Result<DeviceId, String> {
    let parameters = BTreeMap::from([("voltage".to_string(), 0.0)]);
    builder
        .device(
            format!("VAVR{pin}"),
            DeviceKind::VoltageSource,
            vec![node, ground],
            parameters,
        )
        .map_err(|error| error.to_string())
}

/// Solves the DC operating point of an extracted netlist.
pub(crate) fn solve_dc(netlist: &Netlist) -> Result<DcResult, String> {
    let circuit = build_circuit(netlist)?;
    dc_operating_point(&circuit, &DcConfig::default()).map_err(|error| error.to_string())
}

/// Returns a terminal current for each netlist component at a DC solution.
pub(crate) fn device_currents(
    netlist: &Netlist,
    result: &DcResult,
) -> Result<Vec<(String, Option<f64>)>, String> {
    let circuit = build_circuit(netlist)?;
    let currents = dc_terminal_currents(&circuit, result).map_err(|error| error.to_string())?;
    if currents.len() != netlist.components.len() {
        return Err("device current count mismatch".to_string());
    }
    Ok(netlist
        .components
        .iter()
        .zip(currents)
        .map(|(component, current)| (component.label.clone(), current))
        .collect())
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
