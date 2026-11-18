//! Deterministic digital to analog co-simulation with analog feedback.

#![forbid(unsafe_code)]

use analysis::{TransientConfig, TransientSession};
use cells::catalog_digital::DigitalEntry;
use kernel::{Circuit, DeviceId, NodeId, SimError, Time};

/// Analog input fed back to one digital input.
#[derive(Clone, Copy)]
pub struct PinInput {
    /// Digital input name.
    pub name: &'static str,
    /// Analog voltage threshold.
    pub threshold: f64,
}

/// Digital output and analog source link.
#[derive(Clone, Copy)]
pub struct PinLink {
    /// Digital output name.
    pub output: &'static str,
    /// Analog voltage source device.
    pub source: DeviceId,
    /// Analog node returned to the caller.
    pub node: NodeId,
    /// Source voltage for digital high.
    pub high: f64,
    /// Source voltage for digital low.
    pub low: f64,
    /// Optional analog feedback input.
    pub input: Option<PinInput>,
}

/// A digital circuit driving an analog voltage source.
pub struct MixedSession {
    digital: DigitalEntry,
    analog: TransientSession,
    link: PinLink,
    voltage: f64,
}

impl MixedSession {
    /// Builds a session with a digital entry and a pin link.
    pub fn new(
        circuit: &Circuit,
        config: &TransientConfig,
        digital: DigitalEntry,
        link: PinLink,
    ) -> Result<Self, SimError> {
        if digital.circuit.output(link.output).is_none() {
            return Err(SimError::InvalidNetlist(format!(
                "unknown digital output: {}",
                link.output
            )));
        }
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self {
            digital,
            analog,
            link,
            voltage: 0.0,
        })
    }

    /// Advances one digital tick and one analog step, returning the node voltage.
    pub fn step(&mut self) -> Result<f64, SimError> {
        self.apply_input()?;
        self.digital.circuit.tick()?;
        let value = self.source_value()?;
        self.analog.set_source_voltage(self.link.source, value)?;
        let sample = self.analog.step()?;
        self.voltage = sample.solution.voltage(self.link.node).unwrap_or(0.0);
        Ok(self.voltage)
    }

    fn apply_input(&mut self) -> Result<(), SimError> {
        let Some(input) = self.link.input else {
            return Ok(());
        };
        let value = self.voltage > input.threshold;
        self.digital.circuit.set_input(input.name, value)
    }

    fn source_value(&self) -> Result<f64, SimError> {
        let output = self
            .digital
            .circuit
            .output(self.link.output)
            .ok_or_else(|| {
                SimError::InvalidNetlist(format!("unknown digital output: {}", self.link.output))
            })?;
        Ok(if output {
            self.link.high
        } else {
            self.link.low
        })
    }

    /// Returns the analog simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.analog.time()
    }

    /// Returns the last analog node voltage.
    #[must_use]
    pub fn voltage(&self) -> f64 {
        self.voltage
    }

    /// Returns the current digital output.
    #[must_use]
    pub fn digital_output(&self) -> Option<bool> {
        self.digital.circuit.output(self.link.output)
    }
}

#[cfg(test)]
mod tests {
    use super::{MixedSession, PinInput, PinLink};
    use analysis::TransientConfig;
    use cells::DigitalBuilder;
    use cells::catalog_digital::{DigitalEntry, digital_entry};
    use kernel::{Circuit, CircuitBuilder, DeviceId, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn rc_circuit(resistance: f64, capacitance: f64) -> (Circuit, DeviceId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let source = builder.node("in").expect("node");
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        let source_id = add_source(&mut builder, source, ground);
        add_passives(
            &mut builder,
            source,
            output,
            ground,
            resistance,
            capacitance,
        );
        (builder.build().expect("circuit"), source_id, output)
    }

    fn add_source(builder: &mut CircuitBuilder, source: NodeId, ground: NodeId) -> DeviceId {
        let params = BTreeMap::from([("voltage".to_string(), 0.0)]);
        builder
            .device(
                "V1",
                DeviceKind::VoltageSource,
                vec![source, ground],
                params,
            )
            .expect("V1")
    }

    fn add_passives(
        builder: &mut CircuitBuilder,
        source: NodeId,
        output: NodeId,
        ground: NodeId,
        resistance: f64,
        capacitance: f64,
    ) {
        let resistor = BTreeMap::from([("resistance".to_string(), resistance)]);
        builder
            .device("R1", DeviceKind::Resistor, vec![source, output], resistor)
            .expect("R1");
        let capacitor = BTreeMap::from([("capacitance".to_string(), capacitance)]);
        builder
            .device("C1", DeviceKind::Capacitor, vec![output, ground], capacitor)
            .expect("C1");
    }

    fn clock_entry() -> DigitalEntry {
        digital_entry("source.digital.digital_clock_source")
            .expect("build")
            .expect("clock")
    }

    fn inverter_entry() -> DigitalEntry {
        let mut builder = DigitalBuilder::new();
        let input = builder.input("a");
        let output = builder.not(input);
        builder.output("y", output);
        DigitalEntry {
            id: "test.inverter",
            circuit: builder.build(),
            inputs: vec!["a"],
            outputs: vec!["y"],
        }
    }

    fn config(dt: f64, steps: usize) -> TransientConfig {
        TransientConfig {
            dt,
            steps,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        }
    }

    #[test]
    fn clock_toggles_source_and_preserves_capacitor_state() {
        let (circuit, source, node) = rc_circuit(100.0, 1e-9);
        let link = PinLink {
            output: "q",
            source,
            node,
            high: 5.0,
            low: 0.0,
            input: None,
        };
        let mut session =
            MixedSession::new(&circuit, &config(1e-6, 10), clock_entry(), link).expect("session");
        let high = session.step().expect("high step");
        assert!(high > 4.5, "high={high}");
        let low = session.step().expect("low step");
        assert!(low < 0.5, "low={low}");
        assert_eq!(session.time().as_ps(), 2_000_000);
    }

    #[test]
    fn analog_feedback_drives_digital_inverter() {
        let (circuit, source, node) = rc_circuit(1000.0, 1e-6);
        let link = PinLink {
            output: "y",
            source,
            node,
            high: 5.0,
            low: 0.0,
            input: Some(PinInput {
                name: "a",
                threshold: 2.5,
            }),
        };
        let mut session = MixedSession::new(&circuit, &config(1e-5, 1000), inverter_entry(), link)
            .expect("session");
        let toggles = count_toggles(&mut session, 1000);
        assert!(toggles >= 4, "toggles={toggles}");
    }

    fn count_toggles(session: &mut MixedSession, steps: usize) -> usize {
        let mut toggles = 0;
        let mut previous = session.digital_output();
        for _ in 0..steps {
            let voltage = session.step().expect("step");
            assert!((-0.1..=5.1).contains(&voltage), "voltage={voltage}");
            let current = session.digital_output();
            if current != previous {
                toggles += 1;
                previous = current;
            }
        }
        toggles
    }
}
