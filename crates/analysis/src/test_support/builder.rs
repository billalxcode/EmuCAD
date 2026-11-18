use super::parameters::pulse_parameters;
use super::*;

pub(crate) struct TestCircuit {
    pub(super) builder: CircuitBuilder,
}

impl TestCircuit {
    pub(crate) fn new() -> Self {
        Self {
            builder: CircuitBuilder::new(),
        }
    }

    pub(crate) fn node(&mut self, name: &str) -> NodeId {
        self.builder.node(name).expect(name)
    }

    pub(crate) fn supply(&mut self, name: &str) -> NodeId {
        self.builder.supply(name).expect(name)
    }

    pub(crate) fn ground(&self) -> NodeId {
        self.builder.ground()
    }

    pub(crate) fn voltage(&mut self, name: &str, positive: NodeId, negative: NodeId, value: f64) {
        let parameters = params("voltage", value);
        self.two_pin(
            name,
            DeviceKind::VoltageSource,
            positive,
            negative,
            parameters,
        );
    }

    pub(crate) fn resistor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("resistance", value);
        self.two_pin(name, DeviceKind::Resistor, first, second, parameters);
    }

    pub(crate) fn capacitor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("capacitance", value);
        self.two_pin(name, DeviceKind::Capacitor, first, second, parameters);
    }

    pub(crate) fn inductor(&mut self, name: &str, first: NodeId, second: NodeId, value: f64) {
        let parameters = params("inductance", value);
        self.two_pin(name, DeviceKind::Inductor, first, second, parameters);
    }

    pub(crate) fn current(&mut self, name: &str, from: NodeId, to: NodeId, value: f64) {
        let parameters = params("current", value);
        self.two_pin(name, DeviceKind::CurrentSource, from, to, parameters);
    }

    pub(crate) fn diode(&mut self, name: &str, anode: NodeId, cathode: NodeId) {
        self.two_pin(name, DeviceKind::Diode, anode, cathode, BTreeMap::new());
    }

    pub(crate) fn mosfet(
        &mut self,
        name: &str,
        kind: DeviceKind,
        gate: NodeId,
        drain: NodeId,
        source: NodeId,
        width: f64,
    ) {
        let parameters = mos_params(width);
        self.builder
            .device(name, kind, vec![gate, drain, source], parameters)
            .expect(name);
    }

    pub(crate) fn mosfet_body(
        &mut self,
        name: &str,
        kind: DeviceKind,
        pins: [NodeId; 4],
        width: f64,
    ) {
        let mut parameters = mos_params(width);
        parameters.insert("gamma".to_string(), 0.5);
        parameters.insert("phi".to_string(), 0.7);
        self.builder
            .device(name, kind, pins.to_vec(), parameters)
            .expect(name);
    }

    pub(crate) fn bjt(
        &mut self,
        name: &str,
        kind: DeviceKind,
        collector: NodeId,
        base: NodeId,
        emitter: NodeId,
        beta: f64,
    ) {
        let parameters = params("bf", beta);
        self.builder
            .device(name, kind, vec![collector, base, emitter], parameters)
            .expect(name);
    }

    pub(crate) fn jfet(
        &mut self,
        name: &str,
        kind: DeviceKind,
        drain: NodeId,
        gate: NodeId,
        source: NodeId,
        parameters: (f64, f64),
    ) {
        let mut map = BTreeMap::new();
        map.insert("vto".to_string(), parameters.0);
        map.insert("beta".to_string(), parameters.1);
        self.builder
            .device(name, kind, vec![drain, gate, source], map)
            .expect(name);
    }

    pub(crate) fn pulse(
        &mut self,
        name: &str,
        positive: NodeId,
        negative: NodeId,
        levels: (f64, f64),
        timing: (f64, f64, f64, f64, f64),
    ) {
        let parameters = pulse_parameters(levels, timing);
        self.two_pin(
            name,
            DeviceKind::PulseVoltageSource,
            positive,
            negative,
            parameters,
        );
    }

    pub(crate) fn vcvs(
        &mut self,
        name: &str,
        output: [NodeId; 2],
        control: [NodeId; 2],
        gain: f64,
    ) {
        let parameters = params("gain", gain);
        let pins = vec![output[0], output[1], control[0], control[1]];
        self.builder
            .device(name, DeviceKind::Vcvs, pins, parameters)
            .expect(name);
    }

    pub(crate) fn vccs(&mut self, name: &str, output: [NodeId; 2], control: [NodeId; 2], gm: f64) {
        let parameters = params("gm", gm);
        let pins = vec![output[0], output[1], control[0], control[1]];
        self.builder
            .device(name, DeviceKind::Vccs, pins, parameters)
            .expect(name);
    }

    pub(crate) fn cccs(
        &mut self,
        name: &str,
        output: [NodeId; 2],
        control: [NodeId; 2],
        gain: f64,
    ) {
        let parameters = params("gain", gain);
        let pins = vec![output[0], output[1], control[0], control[1]];
        self.builder
            .device(name, DeviceKind::Cccs, pins, parameters)
            .expect(name);
    }

    pub(crate) fn ccvs(&mut self, name: &str, output: [NodeId; 2], control: [NodeId; 2], rm: f64) {
        let parameters = params("rm", rm);
        let pins = vec![output[0], output[1], control[0], control[1]];
        self.builder
            .device(name, DeviceKind::Ccvs, pins, parameters)
            .expect(name);
    }

    pub(crate) fn switch(
        &mut self,
        name: &str,
        output: [NodeId; 2],
        control: [NodeId; 2],
        parameters: (f64, f64, f64, f64),
    ) {
        let mut map = BTreeMap::new();
        map.insert("ron".to_string(), parameters.0);
        map.insert("roff".to_string(), parameters.1);
        map.insert("vth".to_string(), parameters.2);
        map.insert("width".to_string(), parameters.3);
        let pins = vec![output[0], output[1], control[0], control[1]];
        self.builder
            .device(name, DeviceKind::Switch, pins, map)
            .expect(name);
    }

    pub(crate) fn transmission_line(
        &mut self,
        name: &str,
        ports: [NodeId; 4],
        parameters: (f64, f64),
    ) {
        let mut map = BTreeMap::new();
        map.insert("z0".to_string(), parameters.0);
        map.insert("delay".to_string(), parameters.1);
        self.builder
            .device(name, DeviceKind::TransmissionLine, ports.to_vec(), map)
            .expect(name);
    }

    pub(crate) fn build(self) -> Circuit {
        self.builder.build().expect("circuit")
    }

    pub(super) fn two_pin(
        &mut self,
        name: &str,
        kind: DeviceKind,
        first: NodeId,
        second: NodeId,
        parameters: BTreeMap<String, f64>,
    ) {
        self.builder
            .device(name, kind, vec![first, second], parameters)
            .expect(name);
    }
}
