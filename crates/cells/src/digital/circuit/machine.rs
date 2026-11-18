use super::*;

/// A two-state digital circuit with an ideal clock.
pub struct DigitalCircuit {
    pub(super) values: Vec<bool>,
    pub(super) gates: Vec<Gate>,
    pub(super) dffs: Vec<Dff>,
    pub(super) inputs: BTreeMap<String, NetId>,
    pub(super) outputs: BTreeMap<String, NetId>,
    pub(super) component_gates: Vec<Vec<usize>>,
    pub(super) orders: Vec<Vec<usize>>,
    pub(super) cyclic: Vec<bool>,
    pub(super) fanout: Vec<Vec<usize>>,
    pub(super) gate_component: Vec<usize>,
    pub(super) gate_rank: Vec<u32>,
    pub(super) dirty: Vec<bool>,
    pub(super) dirty_bits: Vec<Vec<u64>>,
}

/// Static size information for a digital circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircuitStats {
    pub gates: usize,
    pub components: usize,
    pub dffs: usize,
    pub largest_component: usize,
    pub cyclic_components: usize,
}

impl DigitalCircuit {
    /// Returns static circuit size information.
    #[must_use]
    pub fn stats(&self) -> CircuitStats {
        CircuitStats {
            gates: self.gates.len(),
            components: self.component_gates.len(),
            dffs: self.dffs.len(),
            largest_component: self.component_gates.iter().map(Vec::len).max().unwrap_or(0),
            cyclic_components: self.cyclic.iter().filter(|value| **value).count(),
        }
    }

    /// Sets a net value without settling.
    pub fn set_net(&mut self, net: NetId, value: bool) {
        let slot = &mut self.values[net.raw()];
        if *slot == value {
            return;
        }
        *slot = value;
        self.mark_fanout(net);
    }

    fn mark_fanout(&mut self, net: NetId) {
        for index in 0..self.fanout[net.raw()].len() {
            let gate = self.fanout[net.raw()][index];
            let component = self.gate_component[gate];
            let rank = self.gate_rank[gate] as usize;
            self.dirty_bits[component][rank / 64] |= 1 << (rank % 64);
            self.dirty[component] = true;
        }
    }

    pub(super) fn mark_all_dirty(&mut self) {
        self.dirty.fill(true);
        for component in 0..self.orders.len() {
            self.dirty_bits[component] = full_bits(self.orders[component].len());
        }
    }

    /// Sets a named input value without settling.
    pub fn set_input(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        let net = self
            .inputs
            .get(name)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown input: {name}")))?;
        self.set_net(*net, value);
        Ok(())
    }

    /// Reads a named output.
    #[must_use]
    pub fn output(&self, name: &str) -> Option<bool> {
        self.outputs.get(name).map(|net| self.values[net.raw()])
    }

    /// Reads a net value.
    #[must_use]
    pub fn value(&self, net: NetId) -> bool {
        self.values[net.raw()]
    }

    /// Settles combinational logic.
    pub fn settle(&mut self) -> Result<(), SimError> {
        self.settle_cyclic()?;
        for component in 0..self.orders.len() {
            if !self.cyclic[component] {
                self.settle_acyclic(component);
            }
        }
        Ok(())
    }

    fn settle_cyclic(&mut self) -> Result<(), SimError> {
        for component in 0..self.component_gates.len() {
            if !self.dirty[component] || !self.cyclic[component] {
                continue;
            }
            self.settle_component(component)?;
            self.dirty[component] = false;
        }
        Ok(())
    }

    fn settle_acyclic(&mut self, component: usize) {
        for word in 0..self.dirty_bits[component].len() {
            self.settle_word(component, word);
        }
    }

    fn settle_word(&mut self, component: usize, word: usize) {
        loop {
            let mut pending = self.dirty_bits[component][word];
            if pending == 0 {
                return;
            }
            self.dirty_bits[component][word] = 0;
            while pending != 0 {
                let bit = pending.trailing_zeros() as usize;
                pending &= pending - 1;
                self.settle_rank(component, word * 64 + bit);
            }
        }
    }

    fn settle_rank(&mut self, component: usize, rank: usize) {
        let gate = self.orders[component][rank];
        let output = self.gates[gate].output();
        if evaluate(self.gates[gate], &mut self.values) {
            self.mark_fanout(output);
        }
    }

    fn settle_component(&mut self, component: usize) -> Result<(), SimError> {
        for iteration in 0..MAX_SETTLE_ITERATIONS {
            if !self.cyclic_pass(component) {
                self.clear_bits(component);
                return Ok(());
            }
            if iteration + 1 == MAX_SETTLE_ITERATIONS {
                return Err(SimError::ConvergenceFailure { iteration });
            }
        }
        Ok(())
    }

    fn clear_bits(&mut self, component: usize) {
        self.dirty_bits[component].fill(0);
    }

    fn cyclic_pass(&mut self, component: usize) -> bool {
        let mut changed = false;
        for index in 0..self.component_gates[component].len() {
            let gate = self.component_gates[component][index];
            let output = self.gates[gate].output();
            let gate_changed = evaluate(self.gates[gate], &mut self.values);
            if gate_changed {
                self.mark_fanout(output);
            }
            changed |= gate_changed;
        }
        changed
    }

    /// Applies one ideal clock tick and settles.
    pub fn tick(&mut self) -> Result<(), SimError> {
        self.settle()?;
        let next = self.dff_inputs();
        self.update_dffs(&next);
        self.settle()
    }

    /// Clears all flip-flops and settles.
    pub fn reset(&mut self) -> Result<(), SimError> {
        for dff in &self.dffs {
            self.values[dff.output.raw()] = false;
        }
        self.mark_all_dirty();
        self.settle()
    }

    fn dff_inputs(&self) -> Vec<bool> {
        self.dffs
            .iter()
            .map(|dff| self.values[dff.data.raw()])
            .collect()
    }

    fn update_dffs(&mut self, next: &[bool]) {
        for (index, &value) in next.iter().enumerate() {
            let output = self.dffs[index].output;
            self.set_net(output, value);
        }
    }
}

fn full_bits(length: usize) -> Vec<u64> {
    let words = length.div_ceil(64);
    let mut bits = vec![u64::MAX; words];
    let remainder = length % 64;
    if remainder != 0 {
        if let Some(last) = bits.last_mut() {
            *last = (1u64 << remainder) - 1;
        }
    }
    bits
}
