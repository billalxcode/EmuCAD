use super::*;

#[test]
fn transmission_gate_has_ports() {
    let cell = transmission_gate().expect("cell");
    assert!(cell.port("a").is_some());
    assert!(cell.port("b").is_some());
    assert!(cell.port("en").is_some());
    assert!(cell.port("enb").is_some());
}

#[test]
fn tri_state_inverter_has_ports() {
    let cell = tri_state_inverter().expect("cell");
    assert!(cell.port("a").is_some());
    assert!(cell.port("y").is_some());
    assert!(cell.port("en").is_some());
    assert!(cell.port("enb").is_some());
}

#[test]
fn d_latch_captures_input() {
    let cell = d_latch().expect("cell");
    let mut circuit = cell.powered(5.0).expect("powered");
    let data = cell.port("d").expect("d");
    let enable = cell.port("en").expect("en");
    let enable_bar = cell.port("enb").expect("enb");
    add_source(&mut circuit, "vd", data, 5.0).expect("vd");
    add_source(&mut circuit, "ven", enable, 5.0).expect("ven");
    add_source(&mut circuit, "venb", enable_bar, 0.0).expect("venb");
    let output_port = cell.port("q").expect("q");
    assert!(output(&circuit, output_port) > 4.0);
    circuit
        .set_device_parameter("vd", "voltage", 0.0)
        .expect("set data");
    assert!(output(&circuit, output_port) < 1.0);
}

#[test]
fn dff_has_ports() {
    let cell = d_flip_flop().expect("cell");
    assert!(cell.port("d").is_some());
    assert!(cell.port("clk").is_some());
    assert!(cell.port("q").is_some());
}

#[test]
fn sram_has_ports() {
    let cell = sram6t().expect("cell");
    assert!(cell.port("q").is_some());
    assert!(cell.port("qb").is_some());
    assert!(cell.port("wl").is_some());
    assert!(cell.port("bl").is_some());
    assert!(cell.port("blb").is_some());
}

#[test]
fn ring_oscillator_has_output() {
    let cell = ring_oscillator().expect("cell");
    assert!(cell.port("y").is_some());
    assert!(cell.powered(5.0).is_ok());
}

#[test]
fn registry_builds_every_cell() {
    for kind in CellKind::ALL {
        let cell = kind.build().expect("cell");
        assert!(!kind.name().is_empty());
        assert!(cell.vdd().is_some());
    }
}
#[test]
fn inverter_noise_margins_are_positive() {
    let (output_port, points) = inverter_sweep();
    let samples: Vec<(f64, f64)> = points
        .iter()
        .map(|point| {
            (
                point.sweep,
                point.solution.voltage(output_port).unwrap_or(0.0),
            )
        })
        .collect();
    let metrics = noise_margins(&samples).expect("metrics");
    assert!(metrics.voh > 4.0);
    assert!(metrics.vol < 1.0);
    assert!(metrics.nmh > 1.0);
    assert!(metrics.nml > 1.0);
}
