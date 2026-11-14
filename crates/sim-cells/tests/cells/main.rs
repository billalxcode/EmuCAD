use sim_analysis::{DcConfig, dc_operating_point, dc_sweep};
use sim_cells::{
    CellKind, adder8, alu_slice, and2, buffer, d_flip_flop, d_latch, full_adder, half_adder,
    inverter, mux2, nand2, noise_margins, nor2, or2, ring_oscillator, ripple_carry_adder, sram6t,
    transmission_gate, tri_state_inverter, xor2,
};
use sim_core::{Circuit, DeviceKind, NodeId, SimError};
use std::collections::BTreeMap;

fn params(name: &str, value: f64) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    map.insert(name.to_string(), value);
    map
}

fn add_source(
    circuit: &mut Circuit,
    name: &str,
    node: NodeId,
    voltage: f64,
) -> Result<(), SimError> {
    let ground = circuit.ground();
    circuit.add_device(
        name,
        DeviceKind::VoltageSource,
        vec![node, ground],
        params("voltage", voltage),
    )?;
    Ok(())
}

fn output(circuit: &Circuit, node: NodeId) -> f64 {
    let result = dc_operating_point(circuit, &DcConfig::default()).expect("dc");
    result.solution.voltage(node).expect("output")
}

fn inverter_sweep() -> (NodeId, Vec<sim_analysis::DcPoint>) {
    let cell = inverter().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    add_source(&mut circuit, "vin", input, 0.0).expect("source");
    let points = dc_sweep(
        &circuit,
        "vin",
        "voltage",
        0.0,
        5.0,
        0.1,
        &DcConfig::default(),
    )
    .expect("sweep");
    (cell.port("y").expect("y"), points)
}

fn inverter_output(input_voltage: f64) -> f64 {
    let cell = inverter().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let input = cell.port("a").expect("a");
    add_source(&mut circuit, "vin", input, input_voltage).expect("source");
    output(&circuit, cell.port("y").expect("y"))
}

mod adders;
mod alu;
mod logic;
mod sequential;
