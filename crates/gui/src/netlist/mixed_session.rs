//! Mixed AVR and analog session wiring for a GUI netlist.

use super::{Netlist, build_circuit_with_sources};
use analysis::TransientConfig;
use avr::{AvrPackageMap, AvrSignal, PinElectricalConfig};
use kernel::{DeviceId, NodeId};
use mixed::{AdcLink, AvrPin, AvrSession, PwmLink, UartRxLink, UartTxLink};
use std::collections::{BTreeMap, BTreeSet};

/// Analog connections for one mixed session.
pub(crate) struct MixedPinPlan {
    /// GPIO output package pins.
    pub outputs: Vec<u8>,
    /// ADC channel and package pin pairs.
    pub adcs: Vec<(u8, u8)>,
    /// Timer PWM output package pins.
    pub pwms: Vec<u8>,
    /// UART TX package pin.
    pub uart_tx: Option<u8>,
    /// UART RX package pin.
    pub uart_rx: Option<u8>,
    /// Analog high level.
    pub high: f64,
    /// Analog low level.
    pub low: f64,
    /// Thevenin resistance seen by each IO pin.
    pub external_resistance: f64,
    /// ADC reference voltage.
    pub adc_reference: f64,
    /// UART RX threshold voltage.
    pub uart_threshold: f64,
}

/// Builds a mixed AVR and analog session from a netlist.
pub(crate) fn build_mixed_session(
    netlist: &Netlist,
    words: Vec<u16>,
    plan: &MixedPinPlan,
    config: &TransientConfig,
) -> Result<AvrSession, String> {
    validate_plan(plan)?;
    let sources = source_pins(plan)?;
    let (circuit, source_ids) = build_circuit_with_sources(netlist, &sources)?;
    let pins = output_links(netlist, plan, &source_ids)?;
    let adcs = adc_links(netlist, plan)?;
    let pwms = pwm_links(netlist, plan, &source_ids)?;
    let session = AvrSession::with_pwm_links(words, &circuit, config, pins, adcs, pwms)
        .map_err(|error| error.to_string())?;
    attach_uart(session, netlist, plan, &source_ids)
}

fn validate_plan(plan: &MixedPinPlan) -> Result<(), String> {
    if !plan.high.is_finite() || !plan.low.is_finite() || plan.high <= plan.low {
        return Err("mixed output voltage range is invalid".to_string());
    }
    if !plan.external_resistance.is_finite() || plan.external_resistance <= 0.0 {
        return Err("mixed external resistance must be positive".to_string());
    }
    if !plan.adc_reference.is_finite() || plan.adc_reference <= 0.0 {
        return Err("mixed ADC reference must be positive".to_string());
    }
    if !plan.uart_threshold.is_finite() || plan.uart_threshold <= 0.0 {
        return Err("mixed UART threshold must be positive".to_string());
    }
    Ok(())
}

fn source_pins(plan: &MixedPinPlan) -> Result<Vec<u8>, String> {
    let mut pins = BTreeSet::new();
    for &pin in &plan.outputs {
        if !pins.insert(pin) {
            return Err(format!("MCU pin {pin} is listed twice"));
        }
    }
    for &pin in &plan.pwms {
        if !pins.insert(pin) {
            return Err(format!("MCU pin {pin} is already in use"));
        }
    }
    if let Some(pin) = plan.uart_tx {
        if !pins.insert(pin) {
            return Err(format!("MCU pin {pin} is already in use"));
        }
    }
    Ok(pins.into_iter().collect())
}

fn output_links(
    netlist: &Netlist,
    plan: &MixedPinPlan,
    source_ids: &BTreeMap<u8, DeviceId>,
) -> Result<Vec<AvrPin>, String> {
    plan.outputs
        .iter()
        .map(|&pin| output_link(netlist, plan, source_ids, pin))
        .collect()
}

fn output_link(
    netlist: &Netlist,
    plan: &MixedPinPlan,
    source_ids: &BTreeMap<u8, DeviceId>,
    pin: u8,
) -> Result<AvrPin, String> {
    let (port, bit) = mcu_io(netlist, pin)?;
    Ok(AvrPin {
        port,
        bit,
        source: source_for(source_ids, pin)?,
        node: pin_node(netlist, pin)?,
        config: PinElectricalConfig::default(),
        external_resistance: plan.external_resistance,
    })
}

fn adc_links(netlist: &Netlist, plan: &MixedPinPlan) -> Result<Vec<AdcLink>, String> {
    plan.adcs
        .iter()
        .map(|&(channel, pin)| {
            Ok(AdcLink {
                channel,
                node: pin_node(netlist, pin)?,
                reference: plan.adc_reference,
            })
        })
        .collect()
}

fn pwm_links(
    netlist: &Netlist,
    plan: &MixedPinPlan,
    source_ids: &BTreeMap<u8, DeviceId>,
) -> Result<Vec<PwmLink>, String> {
    plan.pwms
        .iter()
        .map(|&pin| {
            mcu_io(netlist, pin)?;
            Ok(PwmLink {
                source: source_for(source_ids, pin)?,
                node: pin_node(netlist, pin)?,
                high: plan.high,
                low: plan.low,
            })
        })
        .collect()
}

fn attach_uart(
    mut session: AvrSession,
    netlist: &Netlist,
    plan: &MixedPinPlan,
    source_ids: &BTreeMap<u8, DeviceId>,
) -> Result<AvrSession, String> {
    if let Some(pin) = plan.uart_tx {
        session = attach_uart_tx(session, netlist, plan, source_ids, pin)?;
    }
    if let Some(pin) = plan.uart_rx {
        let link = UartRxLink {
            node: pin_node(netlist, pin)?,
            threshold: plan.uart_threshold,
        };
        session = session
            .with_uart_rx(link)
            .map_err(|error| error.to_string())?;
    }
    Ok(session)
}

fn attach_uart_tx(
    session: AvrSession,
    netlist: &Netlist,
    plan: &MixedPinPlan,
    source_ids: &BTreeMap<u8, DeviceId>,
    pin: u8,
) -> Result<AvrSession, String> {
    mcu_io(netlist, pin)?;
    let link = UartTxLink {
        source: source_for(source_ids, pin)?,
        node: pin_node(netlist, pin)?,
        high: plan.high,
        low: plan.low,
    };
    session
        .with_uart_tx(link)
        .map_err(|error| error.to_string())
}

fn mcu_io(netlist: &Netlist, pin: u8) -> Result<(u8, u8), String> {
    let mcu = netlist
        .mcu
        .as_ref()
        .ok_or_else(|| "netlist has no MCU".to_string())?;
    if !mcu.pins.contains_key(&pin) {
        return Err(format!("MCU pin {pin} is not mapped"));
    }
    match AvrPackageMap::atmega328p_tqfp32().get(pin) {
        Some(AvrSignal::Io { port, bit }) => Ok((port, bit)),
        _ => Err(format!("MCU pin {pin} is not an IO pin")),
    }
}

fn pin_node(netlist: &Netlist, pin: u8) -> Result<NodeId, String> {
    let mcu = netlist
        .mcu
        .as_ref()
        .ok_or_else(|| "netlist has no MCU".to_string())?;
    let node = mcu
        .pins
        .get(&pin)
        .ok_or_else(|| format!("MCU pin {pin} is not mapped"))?;
    let raw = u32::try_from(*node).map_err(|_| "MCU node index out of range".to_string())?;
    Ok(NodeId::new(raw))
}

fn source_for(source_ids: &BTreeMap<u8, DeviceId>, pin: u8) -> Result<DeviceId, String> {
    source_ids
        .get(&pin)
        .copied()
        .ok_or_else(|| format!("MCU pin {pin} has no source"))
}
