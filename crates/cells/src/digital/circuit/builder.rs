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
        self.dff_in(ClockDomain::DEFAULT, data)
    }

    /// Adds a D flip-flop in one clock domain.
    pub fn dff_in(&mut self, domain: ClockDomain, data: NetId) -> NetId {
        let output = self.new_net();
        self.dff_into_in(domain, data, output);
        output
    }

    /// Adds a D flip-flop that drives an existing net.
    pub fn dff_into(&mut self, data: NetId, output: NetId) {
        self.dff_into_in(ClockDomain::DEFAULT, data, output);
    }

    /// Adds a D flip-flop in one clock domain that drives an existing net.
    pub fn dff_into_in(&mut self, domain: ClockDomain, data: NetId, output: NetId) {
        self.dffs.push(Dff {
            data,
            output,
            domain,
        });
    }

    /// Builds the digital circuit.
    #[must_use]
    pub fn build(self) -> DigitalCircuit {
        let components = build_components(self.values.len(), &self.gates, &self.dffs);
        let mut circuit = assemble(self, components);
        circuit.mark_all_dirty();
        circuit
    }

    pub(crate) fn append_template(
        &mut self,
        template: &DigitalBuilder,
        template_inputs: &[NetId],
        parent_inputs: &[NetId],
        template_outputs: &[NetId],
    ) -> Result<Vec<NetId>, SimError> {
        if template_inputs.len() != parent_inputs.len() {
            return Err(SimError::InvalidNetlist(
                "module input count mismatch".to_string(),
            ));
        }
        let mapping = self.map_nets(template, template_inputs, parent_inputs);
        self.copy_template(template, &mapping);
        Ok(template_outputs
            .iter()
            .map(|net| mapping[net.raw()])
            .collect())
    }

    fn copy_template(&mut self, template: &DigitalBuilder, mapping: &[NetId]) {
        for gate in &template.gates {
            self.gates.push(map_gate(gate, mapping));
        }
        for dff in &template.dffs {
            self.dffs.push(map_dff(dff, mapping));
        }
    }

    fn map_nets(
        &mut self,
        template: &DigitalBuilder,
        template_inputs: &[NetId],
        parent_inputs: &[NetId],
    ) -> Vec<NetId> {
        let mut mapping = Vec::with_capacity(template.values.len());
        for raw in 0..template.values.len() {
            let parent = template_inputs
                .iter()
                .position(|net| net.raw() == raw)
                .map_or_else(|| self.net(), |index| parent_inputs[index]);
            mapping.push(parent);
        }
        mapping
    }

    fn new_net(&mut self) -> NetId {
        let net = NetId(self.values.len());
        self.values.push(false);
        net
    }
}

fn map_gate(gate: &Gate, mapping: &[NetId]) -> Gate {
    match *gate {
        Gate::Const(value, output) => Gate::Const(value, mapping[output.raw()]),
        Gate::Buf(input, output) => Gate::Buf(mapping[input.raw()], mapping[output.raw()]),
        Gate::Not(input, output) => Gate::Not(mapping[input.raw()], mapping[output.raw()]),
        Gate::And(first, second, output) => binary(first, second, output, mapping, Gate::And),
        Gate::Or(first, second, output) => binary(first, second, output, mapping, Gate::Or),
        Gate::Xor(first, second, output) => binary(first, second, output, mapping, Gate::Xor),
        Gate::Nand(first, second, output) => binary(first, second, output, mapping, Gate::Nand),
        Gate::Nor(first, second, output) => binary(first, second, output, mapping, Gate::Nor),
        Gate::Mux {
            first,
            second,
            select,
            output,
        } => map_mux(first, second, select, output, mapping),
    }
}

fn map_mux(first: NetId, second: NetId, select: NetId, output: NetId, mapping: &[NetId]) -> Gate {
    Gate::Mux {
        first: mapping[first.raw()],
        second: mapping[second.raw()],
        select: mapping[select.raw()],
        output: mapping[output.raw()],
    }
}

fn binary(
    first: NetId,
    second: NetId,
    output: NetId,
    mapping: &[NetId],
    gate: fn(NetId, NetId, NetId) -> Gate,
) -> Gate {
    gate(
        mapping[first.raw()],
        mapping[second.raw()],
        mapping[output.raw()],
    )
}

fn map_dff(dff: &Dff, mapping: &[NetId]) -> Dff {
    Dff {
        data: mapping[dff.data.raw()],
        output: mapping[dff.output.raw()],
        domain: dff.domain,
    }
}

impl Default for DigitalBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn assemble(builder: DigitalBuilder, components: CircuitComponents) -> DigitalCircuit {
    let component_count = components.gates.len();
    DigitalCircuit {
        values: builder.values,
        gates: builder.gates,
        dffs: builder.dffs,
        inputs: builder.inputs,
        outputs: builder.outputs,
        component_gates: components.gates,
        orders: components.orders,
        cyclic: components.cyclic,
        fanout: components.fanout,
        gate_component: components.gate_component,
        gate_rank: components.gate_rank,
        dirty: vec![true; component_count],
        dirty_bits: vec![Vec::new(); component_count],
    }
}
