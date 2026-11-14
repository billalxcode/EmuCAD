use super::*;

/// Builds a two-state digital circuit.
pub struct DigitalBuilder {
    values: Vec<bool>,
    gates: Vec<Gate>,
    dffs: Vec<Dff>,
    inputs: BTreeMap<String, NetId>,
    outputs: BTreeMap<String, NetId>,
}

impl DigitalBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            gates: Vec::new(),
            dffs: Vec::new(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }

    /// Adds a named input net.
    pub fn input(&mut self, name: &str) -> NetId {
        let net = self.new_net();
        self.inputs.insert(name.to_string(), net);
        net
    }

    /// Exposes a net as a named output.
    pub fn output(&mut self, name: &str, net: NetId) {
        self.outputs.insert(name.to_string(), net);
    }

    /// Adds an unnamed net.
    pub fn net(&mut self) -> NetId {
        self.new_net()
    }

    /// Adds a constant net.
    pub fn constant(&mut self, value: bool) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Const(value, output));
        output
    }

    /// Adds an inverter.
    pub fn not(&mut self, input: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Not(input, output));
        output
    }

    /// Adds a buffer that drives an existing net.
    pub fn buf_into(&mut self, input: NetId, output: NetId) {
        self.gates.push(Gate::Buf(input, output));
    }

    /// Adds an inverter that drives an existing net.
    pub fn not_into(&mut self, input: NetId, output: NetId) {
        self.gates.push(Gate::Not(input, output));
    }

    /// Adds an AND gate.
    pub fn and(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::And(first, second, output));
        output
    }

    /// Adds an OR gate.
    pub fn or(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Or(first, second, output));
        output
    }

    /// Adds an XOR gate.
    pub fn xor(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Xor(first, second, output));
        output
    }

    /// Adds a NAND gate.
    pub fn nand(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Nand(first, second, output));
        output
    }

    /// Adds a NOR gate.
    pub fn nor(&mut self, first: NetId, second: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Nor(first, second, output));
        output
    }

    /// Adds a two-input multiplexer.
    pub fn mux(&mut self, first: NetId, second: NetId, select: NetId) -> NetId {
        let output = self.new_net();
        self.gates.push(Gate::Mux {
            first,
            second,
            select,
            output,
        });
        output
    }

    /// Adds a D flip-flop that captures data on every clock tick.
    pub fn dff(&mut self, data: NetId) -> NetId {
        let output = self.new_net();
        self.dff_into(data, output);
        output
    }

    /// Adds a D flip-flop that drives an existing net.
    pub fn dff_into(&mut self, data: NetId, output: NetId) {
        self.dffs.push(Dff { data, output });
    }

    /// Builds the digital circuit.
    #[must_use]
    pub fn build(self) -> DigitalCircuit {
        let components = build_components(self.values.len(), &self.gates, &self.dffs);
        let dirty = vec![true; components.gates.len()];
        DigitalCircuit {
            values: self.values,
            gates: self.gates,
            dffs: self.dffs,
            inputs: self.inputs,
            outputs: self.outputs,
            component_gates: components.gates,
            net_component: components.nets,
            dff_component: components.dffs,
            dirty,
        }
    }

    fn new_net(&mut self) -> NetId {
        let net = NetId(self.values.len());
        self.values.push(false);
        net
    }
}

impl Default for DigitalBuilder {
    fn default() -> Self {
        Self::new()
    }
}
