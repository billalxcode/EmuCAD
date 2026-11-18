//! Simulation bindings that connect catalog entries to solver metadata.

use super::{ComponentSpec, ParameterSpec, PinRole, device_parameters, device_pin_roles};
use crate::catalog::catalog;
use kernel::DeviceKind;

/// Fidelity label for a simulation model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fidelity {
    Ideal,
    Compact,
    Behavioral,
    Calibrated,
    Unsupported,
}

impl Fidelity {
    /// Returns a stable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ideal => "ideal",
            Self::Compact => "compact",
            Self::Behavioral => "behavioral",
            Self::Calibrated => "calibrated",
            Self::Unsupported => "unsupported",
        }
    }
}

/// One catalog entry resolved to solver metadata.
#[derive(Clone, Debug)]
pub struct SimulationBinding {
    pub spec: &'static ComponentSpec,
    pub device: DeviceKind,
    pub roles: &'static [PinRole],
    pub parameters: &'static [ParameterSpec],
}

impl SimulationBinding {
    /// Builds a binding when the entry has a solver device.
    #[must_use]
    pub fn of(spec: &'static ComponentSpec) -> Option<Self> {
        let device = spec.device.clone()?;
        Some(Self {
            spec,
            roles: device_pin_roles(&device),
            parameters: device_parameters(&device),
            device,
        })
    }

    /// Returns the pinned role count.
    #[must_use]
    pub fn pin_count(&self) -> usize {
        self.roles.len()
    }

    /// Returns the primary parameter spec when present.
    #[must_use]
    pub fn primary_parameter(&self) -> Option<&'static ParameterSpec> {
        self.parameters
            .iter()
            .find(|parameter| parameter.name == self.spec.parameter)
    }

    /// Returns the fidelity label for this entry.
    #[must_use]
    pub fn fidelity(&self) -> Fidelity {
        fidelity_for(self.spec)
    }

    /// Validates role, pin, and parameter consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.pin_count() != usize::from(self.spec.pins) {
            return Err(format!("{} pin role count mismatch", self.spec.id));
        }
        if self.primary_parameter().is_none() {
            return Err(format!("{} primary parameter missing", self.spec.id));
        }
        let mut labels = std::collections::BTreeSet::new();
        for role in self.roles {
            if !labels.insert(role.label()) {
                return Err(format!("{} duplicate pin role", self.spec.id));
            }
        }
        Ok(())
    }
}

/// Returns one binding by catalog entry ID.
#[must_use]
pub fn simulation_binding(id: &str) -> Option<SimulationBinding> {
    catalog()
        .iter()
        .find(|spec| spec.id == id)
        .and_then(SimulationBinding::of)
}

/// Returns all solver-backed catalog bindings.
pub fn simulation_bindings() -> impl Iterator<Item = SimulationBinding> {
    catalog().iter().filter_map(SimulationBinding::of)
}

fn fidelity_for(spec: &'static ComponentSpec) -> Fidelity {
    if spec.device.is_none() {
        return Fidelity::Unsupported;
    }
    class_fidelity(spec)
}

fn class_fidelity(spec: &'static ComponentSpec) -> Fidelity {
    use super::ModelClass::*;
    match spec.class {
        Resistor | Capacitor | Inductor | VoltageSource | CurrentSource | Vcvs | Vccs | Cccs
        | Ccvs => Fidelity::Ideal,
        Diode | DiodeArray | Led | Photodiode | Bjt | Jfet | Mosfet | OtherTransistor
        | Thyristor | Protection | Switch | Relay | SolidStateRelay | AnalogSwitch | Connector
        | Crystal | Transformer | TransmissionLine | Battery => Fidelity::Compact,
        _ => Fidelity::Behavioral,
    }
}

#[cfg(test)]
mod tests {
    use super::{Fidelity, simulation_binding, simulation_bindings};
    use crate::catalog::catalog;

    #[test]
    fn every_solver_entry_has_binding() {
        for spec in catalog() {
            if spec.device.is_some() {
                assert!(simulation_binding(spec.id).is_some(), "{}", spec.id);
            }
        }
    }

    #[test]
    fn every_binding_validates() {
        for binding in simulation_bindings() {
            binding.validate().unwrap_or_else(|error| panic!("{error}"));
        }
    }

    #[test]
    fn unsupported_entry_has_no_binding() {
        let spec = catalog()
            .iter()
            .find(|spec| spec.device.is_none())
            .expect("unsupported");
        assert!(simulation_binding(spec.id).is_none());
    }

    #[test]
    fn solver_entries_have_fidelity() {
        for binding in simulation_bindings() {
            assert_ne!(
                binding.fidelity(),
                Fidelity::Unsupported,
                "{}",
                binding.spec.id
            );
        }
    }
}
