//! Generic lookup tables and finite state machines.

use super::blocks::{clock_vector, mux_tree};
use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Truth-table lookup with up to ten inputs.
pub fn lookup_table(
    builder: &mut DigitalBuilder,
    inputs: &[NetId],
    truth: &[bool],
) -> Result<NetId, SimError> {
    if inputs.is_empty() || inputs.len() > 10 {
        return Err(invalid("LUT input count must be 1..=10"));
    }
    if truth.len() != 1 << inputs.len() {
        return Err(invalid("LUT truth table size mismatch"));
    }
    let values: Vec<NetId> = truth.iter().map(|&value| builder.constant(value)).collect();
    mux_tree(builder, &values, inputs)
}

/// Specification for a Moore finite state machine.
pub struct FsmSpec<'a> {
    /// Number of state bits.
    pub state_bits: usize,
    /// Number of input bits.
    pub input_bits: usize,
    /// Next-state table indexed by `{state, input}` with input least
    /// significant.
    pub next_state: &'a [u16],
    /// Output table indexed by state.
    pub output: &'a [bool],
}

/// Built state and output of a finite state machine.
pub struct Fsm {
    /// State bits, least significant first.
    pub state: Vec<NetId>,
    /// Moore output for the current state.
    pub output: NetId,
}

/// Builds a Moore finite state machine from truth tables.
pub fn finite_state_machine(
    builder: &mut DigitalBuilder,
    spec: &FsmSpec,
    inputs: &[NetId],
) -> Result<Fsm, SimError> {
    validate_fsm(spec, inputs)?;
    let state: Vec<NetId> = (0..spec.state_bits).map(|_| builder.net()).collect();
    let next = fsm_next_state(builder, spec, inputs, &state)?;
    clock_vector(builder, &state, &next);
    let output = lookup_table(builder, &state, spec.output)?;
    Ok(Fsm { state, output })
}

fn fsm_next_state(
    builder: &mut DigitalBuilder,
    spec: &FsmSpec,
    inputs: &[NetId],
    state: &[NetId],
) -> Result<Vec<NetId>, SimError> {
    let mut select = Vec::with_capacity(inputs.len() + state.len());
    select.extend_from_slice(inputs);
    select.extend_from_slice(state);
    (0..spec.state_bits)
        .map(|bit| {
            let truth: Vec<bool> = spec
                .next_state
                .iter()
                .map(|&value| value & (1 << bit) != 0)
                .collect();
            lookup_table(builder, &select, &truth)
        })
        .collect()
}

fn validate_fsm(spec: &FsmSpec, inputs: &[NetId]) -> Result<(), SimError> {
    validate_fsm_shape(spec, inputs)?;
    validate_fsm_tables(spec)
}

fn validate_fsm_shape(spec: &FsmSpec, inputs: &[NetId]) -> Result<(), SimError> {
    if spec.state_bits == 0 || spec.state_bits > 8 {
        return Err(invalid("FSM state width must be 1..=8"));
    }
    if inputs.len() != spec.input_bits {
        return Err(invalid("FSM input width mismatch"));
    }
    if spec.state_bits + spec.input_bits > 12 {
        return Err(invalid("FSM address width must be at most 12"));
    }
    Ok(())
}

fn validate_fsm_tables(spec: &FsmSpec) -> Result<(), SimError> {
    let total = spec.state_bits + spec.input_bits;
    if spec.next_state.len() != 1 << total {
        return Err(invalid("FSM next-state table size mismatch"));
    }
    if spec
        .next_state
        .iter()
        .any(|&value| value >= 1 << spec.state_bits)
    {
        return Err(invalid("FSM next-state value out of range"));
    }
    if spec.output.len() != 1 << spec.state_bits {
        return Err(invalid("FSM output table size mismatch"));
    }
    Ok(())
}

fn invalid(message: &str) -> SimError {
    SimError::InvalidNetlist(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{FsmSpec, finite_state_machine, lookup_table};
    use crate::digital::{DigitalBuilder, DigitalCircuit, NetId};
    use crate::read_bits;

    fn named_inputs(builder: &mut DigitalBuilder, prefix: &str, width: usize) -> Vec<NetId> {
        (0..width)
            .map(|index| builder.input(&format!("{prefix}{index}")))
            .collect()
    }

    fn set_bits(circuit: &mut DigitalCircuit, prefix: &str, value: u64, width: usize) {
        for index in 0..width {
            let bit = value & (1 << index) != 0;
            circuit
                .set_input(&format!("{prefix}{index}"), bit)
                .expect("input");
        }
    }

    #[test]
    fn lookup_table_matches_truth_table() {
        let mut builder = DigitalBuilder::new();
        let inputs = named_inputs(&mut builder, "a", 3);
        let truth = [false, true, true, false, true, false, false, true];
        let output = lookup_table(&mut builder, &inputs, &truth).expect("lut");
        let mut circuit = builder.build();
        for value in 0..8 {
            set_bits(&mut circuit, "a", value, 3);
            circuit.settle().expect("settle");
            assert_eq!(circuit.value(output), truth[value as usize]);
        }
    }

    fn counter_fsm() -> (DigitalCircuit, NetId) {
        let mut builder = DigitalBuilder::new();
        let spec = FsmSpec {
            state_bits: 2,
            input_bits: 0,
            next_state: &[1, 2, 3, 0],
            output: &[false, false, false, true],
        };
        let fsm = finite_state_machine(&mut builder, &spec, &[]).expect("fsm");
        (builder.build(), fsm.output)
    }

    #[test]
    fn free_running_fsm_cycles_states() {
        let (mut circuit, output) = counter_fsm();
        let expected = [1, 2, 3, 0, 1];
        for state in expected {
            circuit.tick().expect("tick");
            circuit.settle().expect("settle");
            assert_eq!(circuit.value(output), state == 3);
        }
    }

    fn gated_fsm_fixture() -> (DigitalCircuit, NetId) {
        let mut builder = DigitalBuilder::new();
        let input = builder.input("i");
        let spec = FsmSpec {
            state_bits: 1,
            input_bits: 1,
            next_state: &[0, 1, 1, 0],
            output: &[false, true],
        };
        let fsm = finite_state_machine(&mut builder, &spec, &[input]).expect("fsm");
        (builder.build(), fsm.output)
    }

    fn step_fsm(circuit: &mut DigitalCircuit, input: bool, output: NetId) -> bool {
        circuit.set_input("i", input).expect("i");
        circuit.tick().expect("tick");
        circuit.settle().expect("settle");
        circuit.value(output)
    }

    #[test]
    fn gated_fsm_holds_or_toggles() {
        let (mut circuit, output) = gated_fsm_fixture();
        assert!(!step_fsm(&mut circuit, false, output));
        assert!(step_fsm(&mut circuit, true, output));
        assert!(step_fsm(&mut circuit, false, output));
        assert!(!step_fsm(&mut circuit, true, output));
    }

    #[test]
    fn rejects_invalid_lut_and_fsm_shapes() {
        let mut builder = DigitalBuilder::new();
        let inputs = named_inputs(&mut builder, "a", 2);
        assert!(lookup_table(&mut builder, &inputs, &[true]).is_err());
        let spec = FsmSpec {
            state_bits: 2,
            input_bits: 0,
            next_state: &[0, 1, 4],
            output: &[false; 4],
        };
        assert!(finite_state_machine(&mut builder, &spec, &[]).is_err());
    }

    #[test]
    fn fsm_state_tracks_encoding() {
        let mut builder = DigitalBuilder::new();
        let spec = FsmSpec {
            state_bits: 2,
            input_bits: 0,
            next_state: &[1, 2, 3, 0],
            output: &[false; 4],
        };
        let fsm = finite_state_machine(&mut builder, &spec, &[]).expect("fsm");
        let mut circuit = builder.build();
        circuit.tick().expect("tick");
        circuit.tick().expect("tick");
        assert_eq!(read_bits(&circuit, &fsm.state), 2);
    }
}
