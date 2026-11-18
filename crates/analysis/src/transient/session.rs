//! Incremental transient session for pause, step, and reset controls.

use super::steps::{next_time, sample, solve_step, step_time};
use super::validation::validate_config;
use super::*;
use kernel::{Device, DeviceId, DeviceKind};

/// A transient simulation that can be advanced one step at a time.
pub struct TransientSession {
    circuit: Circuit,
    system: SimulationSystem,
    solution: Solution,
    time: Time,
    step: Time,
    config: TransientConfig,
    method: IntegrationMethod,
    newton: NewtonConfig,
    steps: usize,
}

impl TransientSession {
    /// Starts a session with backward Euler.
    pub fn new(circuit: &Circuit, config: &TransientConfig) -> Result<Self, SimError> {
        Self::with_solver(
            circuit,
            config,
            IntegrationMethod::BackwardEuler,
            &NewtonConfig::default(),
        )
    }

    /// Starts a session with an explicit integration method.
    pub fn with_solver(
        circuit: &Circuit,
        config: &TransientConfig,
        method: IntegrationMethod,
        newton: &NewtonConfig,
    ) -> Result<Self, SimError> {
        validate_config(config)?;
        validate_newton(newton)?;
        let mut system = SimulationSystem::new(circuit)?;
        let solution = initial_solution(&mut system, config)?;
        system.update(&solution)?;
        Ok(Self {
            circuit: circuit.clone(),
            system,
            solution,
            time: Time::ZERO,
            step: step_time(config.dt)?,
            config: config.clone(),
            method,
            newton: *newton,
            steps: 0,
        })
    }

    /// Advances one step and returns the sample.
    pub fn step(&mut self) -> Result<TransientSample, SimError> {
        if self.is_finished() {
            return Err(SimError::InvalidNetlist(
                "transient session is finished".to_string(),
            ));
        }
        let (next, residual, iterations) = solve_step(
            &mut self.system,
            &self.solution,
            self.config.dt,
            self.config.gmin,
            self.method,
            &self.newton,
        )?;
        self.system.update(&next)?;
        self.time = next_time(self.time, self.step)?;
        let sample = sample(&self.system, self.time, &next, residual, iterations);
        self.solution = next;
        self.steps += 1;
        Ok(sample)
    }

    /// Resets the session to time zero.
    pub fn reset(&mut self) -> Result<(), SimError> {
        *self = Self::with_solver(&self.circuit, &self.config, self.method, &self.newton)?;
        Ok(())
    }

    /// Updates the voltage of an independent voltage source in place.
    pub fn set_source_voltage(&mut self, device: DeviceId, value: f64) -> Result<(), SimError> {
        let index = self
            .circuit
            .devices()
            .iter()
            .position(|entry| entry.id() == device)
            .ok_or_else(|| SimError::InvalidNetlist(format!("unknown device: {}", device.raw())))?;
        if self.circuit.devices()[index].kind() != &DeviceKind::VoltageSource {
            return Err(SimError::InvalidNetlist(
                "set_source_voltage requires a voltage source".to_string(),
            ));
        }
        self.system.set_parameter(index, "voltage", value)
    }

    /// Returns true when all configured steps are done.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.steps >= self.config.steps
    }

    /// Returns the current simulation time.
    #[must_use]
    pub fn time(&self) -> Time {
        self.time
    }

    /// Returns the completed step count.
    #[must_use]
    pub fn completed_steps(&self) -> usize {
        self.steps
    }

    /// Returns the configured step count.
    #[must_use]
    pub fn total_steps(&self) -> usize {
        self.config.steps
    }
}

#[cfg(test)]
mod tests {
    use super::TransientSession;
    use kernel::{CircuitBuilder, DeviceKind, NodeId};
    use std::collections::BTreeMap;

    fn rc_circuit() -> kernel::Circuit {
        let mut builder = CircuitBuilder::new();
        let source = builder.node("in").expect("node");
        let output = builder.node("out").expect("node");
        let ground = builder.ground();
        add_source(&mut builder, source, ground);
        add_passives(&mut builder, source, output, ground);
        builder.build().expect("circuit")
    }

    fn add_source(builder: &mut CircuitBuilder, source: NodeId, ground: NodeId) {
        let params = BTreeMap::from([("voltage".to_string(), 5.0)]);
        builder
            .device(
                "V1",
                DeviceKind::VoltageSource,
                vec![source, ground],
                params,
            )
            .expect("V1");
    }

    fn add_passives(builder: &mut CircuitBuilder, source: NodeId, output: NodeId, ground: NodeId) {
        let resistor = BTreeMap::from([("resistance".to_string(), 1000.0)]);
        builder
            .device("R1", DeviceKind::Resistor, vec![source, output], resistor)
            .expect("R1");
        let capacitor = BTreeMap::from([("capacitance".to_string(), 1e-6)]);
        builder
            .device("C1", DeviceKind::Capacitor, vec![output, ground], capacitor)
            .expect("C1");
    }

    #[test]
    fn session_steps_and_finishes() {
        let circuit = rc_circuit();
        let config = super::TransientConfig {
            dt: 1e-5,
            steps: 3,
            use_dc_op: true,
            ..super::TransientConfig::default()
        };
        let mut session = TransientSession::new(&circuit, &config).expect("session");
        assert_eq!(session.completed_steps(), 0);
        for _ in 0..3 {
            session.step().expect("step");
        }
        assert!(session.is_finished());
        assert!(session.time().as_ps() > 0);
        assert!(session.step().is_err());
    }

    #[test]
    fn session_resets_to_zero() {
        let circuit = rc_circuit();
        let config = super::TransientConfig {
            dt: 1e-5,
            steps: 2,
            ..super::TransientConfig::default()
        };
        let mut session = TransientSession::new(&circuit, &config).expect("session");
        session.step().expect("step");
        session.reset().expect("reset");
        assert_eq!(session.completed_steps(), 0);
        assert_eq!(session.time().as_ps(), 0);
    }
}
