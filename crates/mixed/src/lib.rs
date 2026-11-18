//! Deterministic one-way digital to analog co-simulation.

#![forbid(unsafe_code)]

use analysis::{TransientConfig, TransientSession};
use cells::catalog_digital::{DigitalEntry, digital_entry};
use kernel::{Circuit, DeviceId, NodeId, SimError, Time};

/// A digital clock driving an analog voltage source.
pub struct MixedSession {
    digital: DigitalEntry,
    analog: TransientSession,
    source: DeviceId,
    output: NodeId,
    high: f64,
    low: f64,
}

impl MixedSession {
    /// Builds a one-way session with a digital clock and an analog RC circuit.
    pub fn new(
        circuit: &Circuit,
        config: &TransientConfig,
        source: DeviceId,
        output: NodeId,
        high: f64,
        low: f64,
    ) -> Result<Self, SimError> {
        let digital = digital_entry("source.digital.digital_clock_source")?
            .ok_or_else(|| SimError::InvalidNetlist("digital clock is unavailable".to_string()))?;
        let analog = TransientSession::new(circuit, config)?;
        Ok(Self {
            digital,
            analog,
            source,
            output,
            high,
            low,
        })
    }

    /// Advances one digital tick and one analog step, returning the output voltage.
    pub fn step(&mut self) -> Result<f64, SimError> {
        self.digital.circuit.tick()?;
        let high = self.digital.circuit.output("q") == Some(true);
        let value = if high { self.high } else { self.low };
        self.analog.set_source_voltage(self.source, value)?;
        let sample = self.analog.step()?;
        Ok(sample.solution.voltage(self.output).unwrap_or(0.0))
    }

    /// Returns the analog simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.analog.time()
    }

    /// Returns the digital clock output.
    #[must_use]
    pub fn clock(&self) -> bool {
        self.digital.circuit.output("q") == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::MixedSession;
    use analysis::TransientConfig;
    use kernel::{Circuit, CircuitBuilder, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn rc_circuit() -> (Circuit, kernel::DeviceId, NodeId) {
        let mut builder = CircuitBuilder::new();
        let source = builder.node("in").expect("node");
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        let source_id = add_source(&mut builder, source, ground);
        add_passives(&mut builder, source, output, ground);
        (builder.build().expect("circuit"), source_id, output)
    }

    fn add_source(
        builder: &mut CircuitBuilder,
        source: NodeId,
        ground: NodeId,
    ) -> kernel::DeviceId {
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

    fn add_passives(builder: &mut CircuitBuilder, source: NodeId, output: NodeId, ground: NodeId) {
        let resistor = BTreeMap::from([("resistance".to_string(), 100.0)]);
        builder
            .device("R1", DeviceKind::Resistor, vec![source, output], resistor)
            .expect("R1");
        let capacitor = BTreeMap::from([("capacitance".to_string(), 1e-9)]);
        builder
            .device("C1", DeviceKind::Capacitor, vec![output, ground], capacitor)
            .expect("C1");
    }

    #[test]
    fn clock_toggles_source_and_preserves_capacitor_state() {
        let (circuit, source, output) = rc_circuit();
        let config = TransientConfig {
            dt: 1e-6,
            steps: 10,
            gmin: 1e-12,
            use_dc_op: false,
            initial_voltages: Vec::new(),
        };
        let mut session =
            MixedSession::new(&circuit, &config, source, output, 5.0, 0.0).expect("session");
        let high = session.step().expect("high step");
        assert!(session.clock());
        assert!(high > 4.5, "high={high}");
        let low = session.step().expect("low step");
        assert!(!session.clock());
        assert!(low < 0.5, "low={low}");
        assert_eq!(session.time().as_ps(), 2_000_000);
    }
}
