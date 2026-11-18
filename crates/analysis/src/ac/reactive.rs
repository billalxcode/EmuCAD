use super::*;

pub(super) struct ReactiveContext<'a> {
    bias: &'a Solution,
    frequency: f64,
    nodes: &'a [Option<usize>],
    branches: &'a [Option<usize>],
}

impl<'a> ReactiveContext<'a> {
    pub(super) fn new(
        bias: &'a Solution,
        frequency: f64,
        nodes: &'a [Option<usize>],
        branches: &'a [Option<usize>],
    ) -> Self {
        Self {
            bias,
            frequency,
            nodes,
            branches,
        }
    }
}

impl ReactiveContext<'_> {
    fn node_voltage(&self, node: NodeId) -> f64 {
        self.bias.voltage(node).unwrap_or(0.0)
    }

    fn stamp(
        &self,
        matrix: &mut ComplexMatrix,
        first: NodeId,
        second: NodeId,
        capacitance: f64,
    ) -> Result<(), SimError> {
        let pair = [
            self.nodes[first.raw() as usize],
            self.nodes[second.raw() as usize],
        ];
        stamp_pair(matrix, pair, reactance(capacitance, self.frequency))
    }
}

pub(super) fn add_reactive(
    circuit: &Circuit,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    for (index, device) in circuit.devices().iter().enumerate() {
        add_device_reactance(device, index, context, matrix)?;
    }
    Ok(())
}

fn add_device_reactance(
    device: &impl Device,
    index: usize,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    match device.kind() {
        DeviceKind::Capacitor => stamp_capacitor(device, context.nodes, matrix, context.frequency),
        DeviceKind::Inductor => {
            stamp_inductor(device, index, context.branches, matrix, context.frequency)
        }
        DeviceKind::Diode => add_diode_capacitance(device, context, matrix),
        DeviceKind::Nmos | DeviceKind::Pmos => add_mosfet_capacitances(device, context, matrix),
        DeviceKind::Npn | DeviceKind::Pnp => add_bjt_capacitances(device, context, matrix),
        DeviceKind::JfetN | DeviceKind::JfetP => add_jfet_capacitances(device, context, matrix),
        _ => Ok(()),
    }
}

fn add_diode_capacitance(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let voltage = context.node_voltage(pins[0]) - context.node_voltage(pins[1]);
    let capacitance = diode_capacitance(device.parameters(), voltage)?;
    context.stamp(matrix, pins[0], pins[1], capacitance)
}

fn add_mosfet_capacitances(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let body = body_pin(pins);
    let (cgs, cgd, cgb) = mosfet_capacitances(
        device.kind(),
        device.parameters(),
        context.node_voltage(pins[0]),
        context.node_voltage(pins[1]),
        context.node_voltage(pins[2]),
        context.node_voltage(body),
    )?;
    context.stamp(matrix, pins[0], pins[2], cgs)?;
    context.stamp(matrix, pins[0], pins[1], cgd)?;
    context.stamp(matrix, pins[0], body, cgb)
}

fn add_bjt_capacitances(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let (cbe, cbc) = bjt_capacitances(
        device.kind(),
        device.parameters(),
        context.node_voltage(pins[1]),
        context.node_voltage(pins[0]),
        context.node_voltage(pins[2]),
    )?;
    context.stamp(matrix, pins[1], pins[2], cbe)?;
    context.stamp(matrix, pins[1], pins[0], cbc)
}

fn add_jfet_capacitances(
    device: &impl Device,
    context: &ReactiveContext<'_>,
    matrix: &mut ComplexMatrix,
) -> Result<(), SimError> {
    let pins = device.pins();
    let (cgs, cgd) = jfet_capacitances(device.parameters())?;
    context.stamp(matrix, pins[1], pins[2], cgs)?;
    context.stamp(matrix, pins[1], pins[0], cgd)
}

fn body_pin(pins: &[NodeId]) -> NodeId {
    if pins.len() == 4 {
        return pins[3];
    }
    pins[2]
}

fn reactance(capacitance: f64, frequency: f64) -> Complex64 {
    Complex64::new(0.0, 2.0 * std::f64::consts::PI * frequency * capacitance)
}

fn stamp_capacitor(
    device: &impl Device,
    nodes: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    let capacitance = parameter(device, "capacitance")?;
    let value = Complex64::new(0.0, 2.0 * std::f64::consts::PI * frequency * capacitance);
    stamp_pair(matrix, node_pair(nodes, device.pins()), value)
}

fn stamp_inductor(
    device: &impl Device,
    index: usize,
    branches: &[Option<usize>],
    matrix: &mut ComplexMatrix,
    frequency: f64,
) -> Result<(), SimError> {
    let inductance = parameter(device, "inductance")?;
    let value = Complex64::new(0.0, -2.0 * std::f64::consts::PI * frequency * inductance);
    let branch = branch_index(branches, index)?;
    add_entry(matrix, Some(branch), Some(branch), value)
}
