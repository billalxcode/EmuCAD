//! Maps catalog model classes to SVG base symbols.

use sim_device::CatalogRecord;

/// Returns the base asset ID for one catalog record.
pub(crate) fn base_for(record: &CatalogRecord) -> &'static str {
    if let Some(base) = passive_base(record) {
        return base;
    }
    if let Some(base) = source_base(record) {
        return base;
    }
    if let Some(base) = semiconductor_base(record) {
        return base;
    }
    if let Some(base) = analog_base(record) {
        return base;
    }
    if let Some(base) = digital_base(record) {
        return base;
    }
    power_base(record)
}

fn passive_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "Resistor" => Some(resistor_base(record.name)),
        "Capacitor" => Some(capacitor_base(record.name)),
        "Inductor" => Some(inductor_base(record.name)),
        "Transformer" => Some(transformer_base(record.name)),
        "Crystal" => Some("crystal"),
        "TransmissionLine" => Some("transmission_line"),
        _ => None,
    }
}

fn source_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "VoltageSource" => Some("voltage_source"),
        "CurrentSource" => Some("current_source"),
        "Battery" => Some("battery"),
        "BehavioralSource" => Some("behavioral"),
        "DigitalSource" => Some("square_source"),
        "Vcvs" | "Vccs" | "Cccs" | "Ccvs" => Some("controlled_source"),
        _ => None,
    }
}

fn semiconductor_base(record: &CatalogRecord) -> Option<&'static str> {
    if let Some(base) = diode_class_base(record) {
        return Some(base);
    }
    transistor_class_base(record)
}

fn diode_class_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "Diode" => Some(diode_base(record.name)),
        "DiodeArray" => Some("diode_array"),
        "Led" => Some(led_base(record.name)),
        "Photodiode" => Some("photodiode"),
        "Optocoupler" => Some("optocoupler"),
        _ => None,
    }
}

fn transistor_class_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "Bjt" => Some(bjt_base(record)),
        "Jfet" => Some(jfet_base(record)),
        "Mosfet" => Some(mosfet_base(record)),
        "OtherTransistor" => Some("igbt"),
        "Thyristor" => Some(thyristor_base(record.name)),
        "Protection" => Some(protection_base(record.name)),
        "Switch" => Some(switch_base(record.name)),
        "Relay" => Some(relay_base(record.name)),
        "SolidStateRelay" => Some("optocoupler"),
        "AnalogSwitch" => Some("switch_spdt"),
        "Connector" => Some("connector"),
        _ => None,
    }
}

fn analog_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "OpAmp" => Some("opamp"),
        "Comparator" => Some("comparator"),
        "Timer" | "VoltageReference" => Some("ic"),
        _ => None,
    }
}

fn digital_base(record: &CatalogRecord) -> Option<&'static str> {
    match record.model_class.as_str() {
        "LogicGate" => Some(logic_gate_base(record.variant)),
        "LogicGateSpecial" => Some("logic_buffer"),
        "FlipFlop" => Some("flip_flop"),
        "Memory" => Some("ic"),
        "Mcu" => Some("mcu"),
        "Adc" => Some("adc"),
        "Dac" => Some("dac"),
        "Counter" | "Multiplexer" | "Adder" | "LevelShifter" | "ProgrammableLogic" => Some("ic"),
        "SerialInterface" | "TimerPeripheral" => Some("ic"),
        _ => None,
    }
}

fn power_base(record: &CatalogRecord) -> &'static str {
    match record.model_class.as_str() {
        "Regulator" => "regulator",
        "DcDc" | "ChargePump" => "dcdc",
        "Supervisor" | "PowerSwitch" | "MotorDriver" => "ic",
        "Sensor" => "sensor",
        "Actuator" => actuator_base(record.name),
        "Display" => "display",
        "RfBlock" => rf_base(record.name),
        "BehavioralBlock" => "behavioral",
        "BoardModule" => "board",
        _ => "ic",
    }
}

fn resistor_base(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        value if value.contains("thermistor") => "thermistor",
        value if value.contains("potentiometer") => "potentiometer",
        value if value.contains("photoresistor") => "photoresistor",
        value if value.contains("varistor") => "varistor",
        value if value.contains("memristor") => "memristor",
        value if value.contains("fuse") => "fuse",
        value if value.contains("ferrite") => "inductor_ferrite",
        _ => "resistor",
    }
}

fn capacitor_base(name: &str) -> &'static str {
    if contains(name, "electrolytic") || contains(name, "tantalum") {
        return "capacitor_polarized";
    }
    if contains(name, "variable") {
        return "capacitor_variable";
    }
    if contains(name, "trimmer") {
        return "capacitor_trimmer";
    }
    "capacitor"
}

fn inductor_base(name: &str) -> &'static str {
    if contains(name, "ferrite") || contains(name, "bead") {
        return "inductor_ferrite";
    }
    if contains(name, "variable") {
        return "inductor_variable";
    }
    if contains(name, "coupled") || contains(name, "transformer") || contains(name, "choke") {
        return "transformer";
    }
    "inductor"
}

fn transformer_base(name: &str) -> &'static str {
    if contains(name, "center") {
        return "transformer_center";
    }
    "transformer"
}

fn diode_base(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        value if value.contains("zener") => "diode_zener",
        value if value.contains("schottky") => "diode_schottky",
        value if value.contains("tunnel") => "diode_tunnel",
        value if value.contains("varactor") || value.contains("varicap") => "diode_varicap",
        value if value.contains("laser") => "diode_laser",
        value if value.contains("bridge") || value.contains("array") => "diode_array",
        _ => "diode",
    }
}

fn led_base(name: &str) -> &'static str {
    if contains(name, "laser") {
        return "diode_laser";
    }
    "led"
}

fn bjt_base(record: &CatalogRecord) -> &'static str {
    if device_contains(record, "pnp") {
        return "bjt_pnp";
    }
    "bjt_npn"
}

fn jfet_base(record: &CatalogRecord) -> &'static str {
    if device_contains(record, "jfetp") || contains(record.name, "p jfet") {
        return "jfet_p";
    }
    "jfet_n"
}

fn mosfet_base(record: &CatalogRecord) -> &'static str {
    if device_contains(record, "pmos") || contains(record.name, "p mosfet") {
        return "mosfet_p";
    }
    "mosfet_n"
}

fn thyristor_base(name: &str) -> &'static str {
    if contains(name, "triac") {
        return "triac";
    }
    if contains(name, "diac") {
        return "diac";
    }
    "scr"
}

fn protection_base(name: &str) -> &'static str {
    if contains(name, "varistor") {
        return "varistor";
    }
    if contains(name, "tvs") || contains(name, "esd") {
        return "diode";
    }
    "fuse"
}

fn switch_base(name: &str) -> &'static str {
    if contains(name, "pushbutton") {
        return "button";
    }
    if contains(name, "spdt") {
        return "switch_spdt";
    }
    "switch"
}

fn relay_base(name: &str) -> &'static str {
    if contains(name, "spdt") || contains(name, "latching") {
        return "relay_spdt";
    }
    "relay"
}

fn logic_gate_base(variant: u8) -> &'static str {
    match variant {
        0 => "logic_and",
        1 => "logic_or",
        2 => "logic_not",
        3 => "logic_nand",
        4 => "logic_nor",
        5 => "logic_xor",
        6 => "logic_xnor",
        7 => "logic_buffer",
        8 => "logic_schmitt",
        _ => "logic_buffer",
    }
}

fn actuator_base(name: &str) -> &'static str {
    if contains(name, "buzzer") || contains(name, "piezo") {
        return "buzzer";
    }
    if contains(name, "speaker") {
        return "speaker";
    }
    if contains(name, "motor") || contains(name, "solenoid") {
        return "motor";
    }
    actuator_visual_base(name)
}

fn actuator_visual_base(name: &str) -> &'static str {
    if contains(name, "led") {
        return "led";
    }
    if contains(name, "lamp") || contains(name, "neon") {
        return "lamp";
    }
    if contains(name, "heater") {
        return "resistor";
    }
    if contains(name, "peltier") || contains(name, "thermoelectric") {
        return "ic";
    }
    "lamp"
}

fn rf_base(name: &str) -> &'static str {
    if contains(name, "antenna") {
        return "antenna";
    }
    "rf"
}

fn device_contains(record: &CatalogRecord, needle: &str) -> bool {
    record
        .device
        .as_deref()
        .is_some_and(|device| contains(device, needle))
}

fn contains(name: &str, needle: &str) -> bool {
    name.to_ascii_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use super::base_for;
    use crate::symbols::assets;
    use sim_device::catalog_records;

    #[test]
    fn every_catalog_entry_has_a_base_asset() {
        for record in catalog_records() {
            let base = base_for(&record);
            assert!(assets::contains(base), "{} -> {}", record.id, base);
        }
    }
}
