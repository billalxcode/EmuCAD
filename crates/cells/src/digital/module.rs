//! Reusable digital modules with named ports.

use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// A reusable module that can be instantiated into a flat circuit.
pub struct DigitalModule {
    template: DigitalBuilder,
    inputs: Vec<NetId>,
    outputs: Vec<NetId>,
}

impl DigitalModule {
    /// Builds a module with input and output port counts.
    pub fn new<F>(inputs: usize, outputs: usize, build: F) -> Result<Self, SimError>
    where
        F: FnOnce(&mut DigitalBuilder, &[NetId], &[NetId]) -> Result<(), SimError>,
    {
        let mut template = DigitalBuilder::new();
        let input_nets: Vec<NetId> = (0..inputs).map(|_| template.net()).collect();
        let output_nets: Vec<NetId> = (0..outputs).map(|_| template.net()).collect();
        build(&mut template, &input_nets, &output_nets)?;
        Ok(Self {
            template,
            inputs: input_nets,
            outputs: output_nets,
        })
    }

    /// Instantiates the module into a parent builder.
    pub fn instantiate(
        &self,
        parent: &mut DigitalBuilder,
        inputs: &[NetId],
    ) -> Result<Vec<NetId>, SimError> {
        parent.append_template(&self.template, &self.inputs, inputs, &self.outputs)
    }

    /// Returns the number of input ports.
    #[must_use]
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Returns the number of output ports.
    #[must_use]
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::DigitalModule;
    use crate::DigitalBuilder;

    fn inverter() -> DigitalModule {
        DigitalModule::new(1, 1, |builder, inputs, outputs| {
            let inverted = builder.not(inputs[0]);
            builder.buf_into(inverted, outputs[0]);
            Ok(())
        })
        .expect("module")
    }

    #[test]
    fn instantiates_independent_inverters() {
        let module = inverter();
        let mut builder = DigitalBuilder::new();
        let first = builder.input("a");
        let second = builder.input("b");
        let first_out = module.instantiate(&mut builder, &[first]).expect("a");
        let second_out = module.instantiate(&mut builder, &[second]).expect("b");
        builder.output("ya", first_out[0]);
        builder.output("yb", second_out[0]);
        let mut circuit = builder.build();
        circuit.set_input("a", true).expect("a");
        circuit.set_input("b", false).expect("b");
        circuit.settle().expect("settle");
        assert_eq!(circuit.output("ya"), Some(false));
        assert_eq!(circuit.output("yb"), Some(true));
        assert_eq!(module.input_count(), 1);
        assert_eq!(module.output_count(), 1);
    }

    #[test]
    fn instantiates_stateful_module() {
        let module = stateful_module();
        let mut builder = DigitalBuilder::new();
        let data = builder.input("d");
        let load = builder.input("load");
        let output = module
            .instantiate(&mut builder, &[data, load])
            .expect("instance");
        builder.output("q", output[0]);
        let mut circuit = builder.build();
        circuit.set_input("d", true).expect("d");
        circuit.set_input("load", true).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(circuit.output("q"), Some(true));
        circuit.set_input("d", false).expect("d");
        circuit.set_input("load", false).expect("load");
        circuit.tick().expect("tick");
        assert_eq!(circuit.output("q"), Some(true));
    }

    fn stateful_module() -> DigitalModule {
        DigitalModule::new(2, 1, |builder, inputs, outputs| {
            let state = builder.net();
            let next = builder.mux(state, inputs[0], inputs[1]);
            builder.dff_into(next, state);
            builder.buf_into(state, outputs[0]);
            Ok(())
        })
        .expect("module")
    }

    #[test]
    fn rejects_wrong_input_count() {
        let module = inverter();
        let mut builder = DigitalBuilder::new();
        assert!(module.instantiate(&mut builder, &[]).is_err());
    }
}
