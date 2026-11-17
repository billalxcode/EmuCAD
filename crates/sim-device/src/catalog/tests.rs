use super::{ComponentRegistry, ComponentRequest, Family, ModelClass, catalog};
use sim_core::{CircuitBuilder, Device, DeviceId, DeviceKind, NodeId};
use std::collections::BTreeSet;

#[test]
fn catalog_has_500_unique_entries() {
    let entries = catalog();
    assert_eq!(entries.len(), 500);
    let ids: BTreeSet<_> = entries.iter().map(|entry| entry.id).collect();
    assert_eq!(ids.len(), 500);
}

#[test]
fn every_entry_validates() {
    for entry in catalog() {
        entry.validate().unwrap_or_else(|error| panic!("{error}"));
    }
}

#[test]
fn ids_are_lowercase_ascii_slugs() {
    for entry in catalog() {
        assert!(
            entry.id.bytes().all(|byte| byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'.'
                || byte == b'_'),
            "bad id: {}",
            entry.id
        );
    }
}

#[test]
fn all_classes_have_entries() {
    let classes: BTreeSet<_> = catalog().iter().map(|entry| entry.class).collect();
    assert_eq!(classes.len(), 60);
}

#[test]
fn family_counts_match_design() {
    assert_family(Family::Passive, 55);
    assert_family(Family::Source, 25);
    assert_family(Family::Diode, 40);
    assert_family(Family::Transistor, 55);
    assert_family(Family::Protection, 18);
    assert_family(Family::Switch, 25);
    assert_family(Family::Analog, 40);
    assert_family(Family::Logic, 60);
    assert_family(Family::Memory, 15);
    assert_family(Family::Mcu, 25);
    assert_family(Family::Power, 30);
    assert_family(Family::Sensor, 35);
    assert_family(Family::Actuator, 20);
    assert_family(Family::Rf, 18);
    assert_family(Family::Behavioral, 25);
    assert_family(Family::Board, 14);
}

fn assert_family(family: Family, expected: usize) {
    let count = catalog()
        .iter()
        .filter(|entry| entry.class.family() == family)
        .count();
    assert_eq!(count, expected, "family {}", family.name());
}

#[test]
fn scalar_responses_are_finite() {
    for entry in catalog() {
        for input in [-10.0, -1.0, 0.0, 1.0, 10.0] {
            assert!(
                entry.class.scalar_response(input).is_finite(),
                "{} at {input}",
                entry.id
            );
        }
    }
}

#[test]
fn every_entry_has_positive_pins_and_default() {
    for entry in catalog() {
        assert!(entry.pins > 0, "{}", entry.id);
        assert!(entry.default.is_finite(), "{}", entry.id);
    }
}

#[test]
fn analog_classes_expose_a_solver_kind() {
    for entry in catalog() {
        if entry.class.requires_device() {
            assert!(entry.device.is_some(), "{}", entry.id);
        }
    }
}

#[test]
fn required_classes_have_devices() {
    for entry in catalog() {
        if entry.class.requires_device() {
            assert!(entry.device.is_some(), "{}", entry.id);
        }
    }
}

#[test]
fn device_kinds_are_solver_supported() {
    for entry in catalog() {
        let Some(device) = &entry.device else {
            continue;
        };
        assert!(supported_device(device), "{}", entry.id);
    }
}

fn supported_device(device: &sim_core::DeviceKind) -> bool {
    supported_passive(device)
        || supported_active(device)
        || supported_sensor(device)
        || supported_actuator(device)
}

fn supported_actuator(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::LedActuator
            | sim_core::DeviceKind::DcMotor
            | sim_core::DeviceKind::Peltier
            | sim_core::DeviceKind::ThermoelectricGenerator
            | sim_core::DeviceKind::DisplayModule
            | sim_core::DeviceKind::RfAmplifier
            | sim_core::DeviceKind::DcDcConverter
            | sim_core::DeviceKind::ChargePump
            | sim_core::DeviceKind::VoltageReference
            | sim_core::DeviceKind::CurrentReference
            | sim_core::DeviceKind::Supervisor
    )
}

fn supported_sensor(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::Ntc
            | sim_core::DeviceKind::Ptc
            | sim_core::DeviceKind::Rtd
            | sim_core::DeviceKind::Thermocouple
            | sim_core::DeviceKind::TemperatureSensor
            | sim_core::DeviceKind::ThermalSwitch
            | sim_core::DeviceKind::Photoresistor
            | sim_core::DeviceKind::PhotodiodeSensor
            | sim_core::DeviceKind::LinearLightSensor
            | sim_core::DeviceKind::HallSwitch
            | sim_core::DeviceKind::LinearHallSensor
            | sim_core::DeviceKind::ReedSwitchSensor
            | sim_core::DeviceKind::MagnetoresistiveSensor
            | sim_core::DeviceKind::FluxgateSensor
            | sim_core::DeviceKind::Accelerometer
            | sim_core::DeviceKind::Gyroscope
            | sim_core::DeviceKind::Imu
            | sim_core::DeviceKind::TiltSensor
            | sim_core::DeviceKind::VibrationSensor
            | sim_core::DeviceKind::StrainGaugeSensor
            | sim_core::DeviceKind::LoadCell
            | sim_core::DeviceKind::PressureSensor
            | sim_core::DeviceKind::ForceSensor
            | sim_core::DeviceKind::HumiditySensor
            | sim_core::DeviceKind::GasSensor
            | sim_core::DeviceKind::PhSensor
            | sim_core::DeviceKind::ElectrochemicalSensor
            | sim_core::DeviceKind::CurrentSenseResistor
            | sim_core::DeviceKind::HallCurrentSensor
            | sim_core::DeviceKind::VoltageDividerSensor
            | sim_core::DeviceKind::InductiveProximitySensor
            | sim_core::DeviceKind::CapacitiveTouchSensor
            | sim_core::DeviceKind::UltrasonicDistanceSensor
    )
}

fn supported_passive(device: &sim_core::DeviceKind) -> bool {
    supported_basic(device) || supported_source(device) || supported_switch(device)
}

fn supported_basic(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::Resistor
            | sim_core::DeviceKind::Capacitor
            | sim_core::DeviceKind::Inductor
            | sim_core::DeviceKind::TransmissionLine
            | sim_core::DeviceKind::Crystal
            | sim_core::DeviceKind::Transformer
    )
}

fn supported_source(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::VoltageSource
            | sim_core::DeviceKind::PulseVoltageSource
            | sim_core::DeviceKind::CurrentSource
            | sim_core::DeviceKind::Vcvs
            | sim_core::DeviceKind::Vccs
            | sim_core::DeviceKind::Cccs
            | sim_core::DeviceKind::Ccvs
    )
}

fn supported_switch(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::Switch
            | sim_core::DeviceKind::IdealSwitch
            | sim_core::DeviceKind::Relay
            | sim_core::DeviceKind::Varistor
            | sim_core::DeviceKind::Fuse
            | sim_core::DeviceKind::Igbt
            | sim_core::DeviceKind::Ujt
            | sim_core::DeviceKind::Put
            | sim_core::DeviceKind::Scr
            | sim_core::DeviceKind::Triac
            | sim_core::DeviceKind::Diac
            | sim_core::DeviceKind::Gto
            | sim_core::DeviceKind::Scs
    )
}

fn supported_active(device: &sim_core::DeviceKind) -> bool {
    matches!(
        device,
        sim_core::DeviceKind::Nmos
            | sim_core::DeviceKind::Pmos
            | sim_core::DeviceKind::Npn
            | sim_core::DeviceKind::Pnp
            | sim_core::DeviceKind::JfetN
            | sim_core::DeviceKind::JfetP
            | sim_core::DeviceKind::Diode
            | sim_core::DeviceKind::DiodeBridge
            | sim_core::DeviceKind::DualDiode
            | sim_core::DeviceKind::Optocoupler
            | sim_core::DeviceKind::OpAmp
            | sim_core::DeviceKind::Comparator
            | sim_core::DeviceKind::Regulator
            | sim_core::DeviceKind::LogicGate
    )
}

#[test]
fn diode_parameters_use_current_units() {
    for entry in catalog() {
        if entry.class != ModelClass::Diode {
            continue;
        }
        assert_eq!(entry.parameter, "is", "{}", entry.id);
        assert_eq!(entry.unit, "A", "{}", entry.id);
        assert!(
            entry.default > 0.0 && entry.default <= 1e-9,
            "{}: {}",
            entry.id,
            entry.default
        );
    }
}

#[test]
fn waveform_sources_map_to_waveform_devices_only() {
    let registry = ComponentRegistry::new();
    let pulse = registry
        .find("source.pulse.pulse_voltage_source")
        .expect("pulse source");
    assert_eq!(pulse.device, Some(DeviceKind::PulseVoltageSource));
    for id in [
        "source.independent.sine_voltage_source",
        "source.independent.sine_current_source",
        "source.pulse.pwl_voltage_source",
        "source.pulse.exponential_voltage_source",
        "source.pulse.voltage_noise_source",
        "source.pulse.current_noise_source",
        "source.pulse.pulse_current_source",
    ] {
        let entry = registry.find(id).unwrap_or_else(|| panic!("missing {id}"));
        assert!(entry.device.is_none(), "{id}");
    }
}

#[test]
fn logic_gate_defaults_include_variant() {
    let registry = ComponentRegistry::new();
    let spec = registry.find("logic.gate.xor_gate").expect("gate");
    let parameters = spec.default_parameters();
    assert_eq!(parameters.get("gate"), Some(&5.0));
    assert_eq!(parameters.get("vth"), Some(&2.5));
}

#[test]
fn behavioral_classification_covers_catalog() {
    let csv = include_str!("../../../../docs/data/behavioral-classification.csv");
    let ids: BTreeSet<_> = csv
        .lines()
        .skip(1)
        .map(|line| line.split(',').next().expect("id"))
        .collect();
    let expected: BTreeSet<_> = catalog()
        .iter()
        .filter(|entry| entry.device.is_none())
        .map(|entry| entry.id)
        .collect();
    assert_eq!(ids, expected);
}

#[test]
fn behavioral_classification_categories_are_valid() {
    let csv = include_str!("../../../../docs/data/behavioral-classification.csv");
    let allowed = [
        "compact-analog",
        "behavioral-electrical",
        "digital",
        "mixed-signal",
        "structural",
        "multidomain",
        "rf",
        "unsupported",
    ];
    for line in csv.lines().skip(1) {
        let category = line.split(',').nth(3).expect("category");
        assert!(allowed.contains(&category), "{line}");
    }
}

#[test]
fn passive_defaults_are_positive() {
    for entry in catalog() {
        if entry.class.family() == Family::Passive {
            assert!(entry.default > 0.0, "{}", entry.id);
        }
    }
}

#[test]
fn catalog_is_deterministic() {
    let first = catalog();
    let second = catalog();
    assert_eq!(first.len(), second.len());
    for (left, right) in first.iter().zip(second.iter()) {
        assert_eq!(left.id, right.id);
        assert_eq!(left.default.to_bits(), right.default.to_bits());
    }
}

#[test]
fn tiers_are_within_design() {
    for entry in catalog() {
        assert!(entry.class.tier() <= 4, "{}", entry.id);
    }
}

#[test]
fn entries_have_distinct_names_per_family() {
    for entry in catalog() {
        let duplicates = catalog()
            .iter()
            .filter(|other| {
                other.class.family() == entry.class.family() && other.name == entry.name
            })
            .count();
        assert_eq!(duplicates, 1, "{}", entry.id);
    }
}

#[test]
fn principles_and_sources_are_present() {
    for entry in catalog() {
        assert!(!entry.class.principle().is_empty(), "{}", entry.id);
        assert!(!entry.class.family().source().is_empty(), "{}", entry.id);
    }
}

#[test]
fn registry_indexes_every_entry() {
    let registry = ComponentRegistry::new();
    for entry in catalog() {
        assert!(registry.find(entry.id).is_some(), "{}", entry.id);
    }
    assert!(registry.find("missing.entry").is_none());
}

#[test]
fn registry_instantiates_and_models_solver_entries() {
    let registry = ComponentRegistry::new();
    for spec in catalog() {
        if spec.device.is_none() {
            continue;
        }
        let pins = vec![NodeId::GROUND; usize::from(spec.pins)];
        let request = ComponentRequest::new(spec.id, spec.name, DeviceId::new(0), pins);
        let instance = registry
            .instantiate(&request)
            .unwrap_or_else(|error| panic!("{}: {error}", spec.id));
        assert_eq!(instance.kind(), spec.device.as_ref().expect("device"));
        assert!(
            registry.model(spec.id, &instance).is_ok(),
            "model {}",
            spec.id
        );
    }
}

#[test]
fn registry_add_to_rejects_wrong_pin_count() {
    let registry = ComponentRegistry::new();
    let mut builder = CircuitBuilder::new();
    let request = ComponentRequest::new(
        "passive.resistor.ideal_resistor",
        "r1",
        DeviceId::new(0),
        vec![NodeId::GROUND],
    );
    assert!(registry.add_to(&mut builder, &request).is_err());
}

#[test]
fn registry_rejects_behavioral_model() {
    let registry = ComponentRegistry::new();
    let entry = catalog()
        .iter()
        .find(|spec| spec.device.is_none())
        .expect("behavioral entry");
    let pins = vec![NodeId::GROUND; usize::from(entry.pins)];
    let request = ComponentRequest::new(entry.id, entry.name, DeviceId::new(0), pins);
    assert!(registry.instantiate(&request).is_err());
    assert!(!registry.is_solver_backed(entry.id));
}

#[test]
fn mct_uses_zero_turn_off_threshold() {
    let entry = catalog()
        .iter()
        .find(|entry| entry.id == "protection.thyristor.mct")
        .expect("mct");
    let parameters = entry.default_parameters();
    assert_eq!(parameters.get("vgt_off"), Some(&0.0));
}

#[test]
fn put_thyristor_maps_to_put_model() {
    let entry = catalog()
        .iter()
        .find(|entry| entry.id == "protection.thyristor.put_thyristor")
        .expect("put");
    assert_eq!(entry.device, Some(sim_core::DeviceKind::Put));
    assert_eq!(entry.parameter, "vd");
}
