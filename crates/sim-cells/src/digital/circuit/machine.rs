use super::*;

/// A two-state digital circuit with an ideal clock.
pub struct DigitalCircuit {
    pub(super) values: Vec<bool>,
    pub(super) gates: Vec<Gate>,
    pub(super) dffs: Vec<Dff>,
    pub(super) inputs: BTreeMap<String, NetId>,
    pub(super) outputs: BTreeMap<String, NetId>,
    pub(super) component_gates: Vec<Vec<usize>>,
    pub(super) net_component: Vec<Option<usize>>,
    pub(super) dff_component: Vec<Option<usize>>,
    pub(super) dirty: Vec<bool>,
}

impl DigitalCircuit {
    /// Sets a named input value without settling.
    pub fn set_input(&mut self, name: &str, value: bool) -> Result<(), SimError> {
        let net = self
            .inputs
            .get(name)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown input: {name}")))?;
        if self.values[net.raw()] == value {
            return Ok(());
        }
        self.values[net.raw()] = value;
        if let Some(component) = self.net_component[net.raw()] {
            self.dirty[component] = true;
        }
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
        for component in 0..self.component_gates.len() {
            if !self.dirty[component] {
                continue;
            }
            self.settle_component(component)?;
            self.dirty[component] = false;
        }
        Ok(())
    }

    fn settle_component(&mut self, component: usize) -> Result<(), SimError> {
        for iteration in 0..MAX_SETTLE_ITERATIONS {
            let mut changed = false;
            for index in 0..self.component_gates[component].len() {
                let gate = self.component_gates[component][index];
                changed |= evaluate(self.gates[gate], &mut self.values);
            }
            if !changed {
                return Ok(());
            }
            if iteration + 1 == MAX_SETTLE_ITERATIONS {
                return Err(SimError::ConvergenceFailure { iteration });
            }
        }
        Ok(())
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
        self.dirty.fill(true);
        self.settle()
    }

    fn dff_inputs(&self) -> Vec<bool> {
        self.dffs
            .iter()
            .map(|dff| self.values[dff.data.raw()])
            .collect()
    }

    fn update_dffs(&mut self, next: &[bool]) {
        for (index, (dff, value)) in self.dffs.iter().zip(next).enumerate() {
            if self.values[dff.output.raw()] == *value {
                continue;
            }
            self.values[dff.output.raw()] = *value;
            if let Some(component) = self.dff_component[index] {
                self.dirty[component] = true;
            }
        }
    }
}
