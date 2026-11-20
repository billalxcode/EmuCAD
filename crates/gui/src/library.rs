//! Built-in component library.

use crate::model::{Category, ComponentDefinition, GridPos, GridSize, PinDefinition, SymbolKind};
use device::{CatalogRecord, catalog_records};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const RESISTOR_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "1",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "2",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const LED_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "A",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "K",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const BUTTON_PINS: &[PinDefinition] = &[
    PinDefinition {
        name: "1",
        number: "1",
        offset: GridPos::new(0, 1),
    },
    PinDefinition {
        name: "2",
        number: "2",
        offset: GridPos::new(4, 1),
    },
];

const VCC_PINS: &[PinDefinition] = &[PinDefinition {
    name: "VCC",
    number: "1",
    offset: GridPos::new(1, 2),
}];

const GND_PINS: &[PinDefinition] = &[PinDefinition {
    name: "GND",
    number: "1",
    offset: GridPos::new(1, 0),
}];

const MCU_PINS: &[PinDefinition] = &[
    mcu_pin("1", "PD3", 0, 2),
    mcu_pin("2", "PD4", 0, 4),
    mcu_pin("3", "GND1", 0, 6),
    mcu_pin("4", "VCC1", 0, 8),
    mcu_pin("5", "GND2", 0, 10),
    mcu_pin("6", "VCC2", 0, 12),
    mcu_pin("7", "PB6", 0, 14),
    mcu_pin("8", "PB7", 0, 16),
    mcu_pin("9", "PD5", 0, 18),
    mcu_pin("10", "PD6", 0, 20),
    mcu_pin("11", "PD7", 0, 22),
    mcu_pin("12", "PB0", 0, 24),
    mcu_pin("13", "PB1", 0, 26),
    mcu_pin("14", "PB2", 0, 28),
    mcu_pin("15", "PB3", 0, 30),
    mcu_pin("16", "PB4", 0, 32),
    mcu_pin("17", "PB5", 10, 32),
    mcu_pin("18", "AVCC", 10, 30),
    mcu_pin("19", "ADC6", 10, 28),
    mcu_pin("20", "AREF", 10, 26),
    mcu_pin("21", "GND3", 10, 24),
    mcu_pin("22", "ADC7", 10, 22),
    mcu_pin("23", "PC0", 10, 20),
    mcu_pin("24", "PC1", 10, 18),
    mcu_pin("25", "PC2", 10, 16),
    mcu_pin("26", "PC3", 10, 14),
    mcu_pin("27", "PC4", 10, 12),
    mcu_pin("28", "PC5", 10, 10),
    mcu_pin("29", "PC6/RESET", 10, 8),
    mcu_pin("30", "PD0", 10, 6),
    mcu_pin("31", "PD1", 10, 4),
    mcu_pin("32", "PD2", 10, 2),
];

const DEFINITIONS: &[ComponentDefinition] = &[
    ComponentDefinition {
        id: "resistor",
        name: "Resistor",
        category: Category::Passive,
        reference_prefix: "R",
        default_value: "220",
        size: GridSize::new(4, 2),
        pins: RESISTOR_PINS,
        symbol: SymbolKind::Resistor,
    },
    ComponentDefinition {
        id: "led",
        name: "LED",
        category: Category::Passive,
        reference_prefix: "D",
        default_value: "LED",
        size: GridSize::new(4, 2),
        pins: LED_PINS,
        symbol: SymbolKind::Led,
    },
    ComponentDefinition {
        id: "button",
        name: "Button",
        category: Category::Passive,
        reference_prefix: "SW",
        default_value: "TACT",
        size: GridSize::new(4, 2),
        pins: BUTTON_PINS,
        symbol: SymbolKind::Button,
    },
    ComponentDefinition {
        id: "vcc",
        name: "VCC",
        category: Category::Power,
        reference_prefix: "PWR",
        default_value: "5V",
        size: GridSize::new(2, 2),
        pins: VCC_PINS,
        symbol: SymbolKind::Vcc,
    },
    ComponentDefinition {
        id: "gnd",
        name: "GND",
        category: Category::Power,
        reference_prefix: "GND",
        default_value: "0V",
        size: GridSize::new(2, 2),
        pins: GND_PINS,
        symbol: SymbolKind::Gnd,
    },
    ComponentDefinition {
        id: "atmega328p",
        name: "ATmega328P",
        category: Category::Board,
        reference_prefix: "U",
        default_value: "ATmega328P",
        size: GridSize::new(10, 34),
        pins: MCU_PINS,
        symbol: SymbolKind::Mcu,
    },
];

/// Returns the built-in component definitions.
#[must_use]
pub fn builtin() -> &'static [ComponentDefinition] {
    BUILTINS.get_or_init(build_definitions).as_ref()
}

/// Finds a definition by ID.
#[must_use]
pub fn find(id: &str) -> Option<&'static ComponentDefinition> {
    INDEX
        .get_or_init(|| {
            builtin()
                .iter()
                .map(|definition| (definition.id, definition))
                .collect()
        })
        .get(id)
        .copied()
}

// Definitions live for the process lifetime. Leaking generated pins and
// default values keeps the existing &'static ComponentDefinition API.
static BUILTINS: OnceLock<Box<[ComponentDefinition]>> = OnceLock::new();
static INDEX: OnceLock<BTreeMap<&'static str, &'static ComponentDefinition>> = OnceLock::new();

fn build_definitions() -> Box<[ComponentDefinition]> {
    let mut definitions = DEFINITIONS.to_vec();
    definitions.extend(catalog_records().iter().enumerate().map(|(index, record)| {
        catalog_definition(record, u16::try_from(index).expect("catalog fits u16"))
    }));
    definitions.into_boxed_slice()
}

fn catalog_definition(record: &CatalogRecord, variant: u16) -> ComponentDefinition {
    let (size, pins) = pin_layout(record.pins);
    ComponentDefinition {
        id: record.id,
        name: record.name,
        category: Category::from_family(record.family),
        reference_prefix: reference_prefix(record),
        default_value: default_value(record),
        size,
        pins: leak_pins(pins),
        symbol: SymbolKind::Catalog {
            base: crate::symbols::base_for(record),
            variant,
        },
    }
}

fn pin_layout(count: u8) -> (GridSize, Vec<PinDefinition>) {
    match count {
        1 => (GridSize::new(2, 2), vec![pin(1, 1, 0)]),
        2 => (GridSize::new(4, 2), vec![pin(1, 0, 1), pin(2, 4, 1)]),
        3 => (
            GridSize::new(4, 4),
            vec![pin(1, 0, 1), pin(2, 4, 2), pin(3, 0, 3)],
        ),
        4 => (
            GridSize::new(4, 4),
            vec![pin(1, 0, 1), pin(2, 4, 1), pin(3, 0, 3), pin(4, 4, 3)],
        ),
        _ => grid_layout(count),
    }
}

fn grid_layout(count: u8) -> (GridSize, Vec<PinDefinition>) {
    let left = usize::from(count).div_ceil(2);
    let right = usize::from(count) / 2;
    let height = (left * 2 + 2) as i32;
    let mut pins = Vec::with_capacity(usize::from(count));
    for row in 0..left {
        pins.push(pin((row + 1) as u8, 0, (row * 2 + 2) as i32));
    }
    for row in 0..right {
        pins.push(pin((left + row + 1) as u8, 8, (row * 2 + 2) as i32));
    }
    (GridSize::new(8, height), pins)
}

fn pin(index: u8, x: i32, y: i32) -> PinDefinition {
    let label = pin_label(index);
    PinDefinition {
        name: label,
        number: label,
        offset: GridPos::new(x, y),
    }
}

const fn mcu_pin(number: &'static str, name: &'static str, x: i32, y: i32) -> PinDefinition {
    PinDefinition {
        name,
        number,
        offset: GridPos::new(x, y),
    }
}

fn pin_label(index: u8) -> &'static str {
    const LABELS: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];
    LABELS
        .get(usize::from(index).saturating_sub(1))
        .copied()
        .unwrap_or_else(|| Box::leak(index.to_string().into_boxed_str()))
}

fn leak_pins(pins: Vec<PinDefinition>) -> &'static [PinDefinition] {
    Box::leak(pins.into_boxed_slice())
}

fn default_value(record: &CatalogRecord) -> &'static str {
    let value = format_number(record.default);
    let text = format!("{value} {}", record.unit);
    Box::leak(text.into_boxed_str())
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value.abs() < 0.001 || value.abs() >= 1_000_000.0 {
        return format!("{value:e}");
    }
    format!("{value}")
}

fn reference_prefix(record: &CatalogRecord) -> &'static str {
    match record.family {
        "passive" => passive_prefix(record),
        "source" => source_prefix(record),
        "diode" => diode_prefix(record),
        "transistor" => "Q",
        "protection" => protection_prefix(record),
        "switch" => switch_prefix(record),
        "analog" | "logic" | "memory" | "mcu" | "power" => "U",
        "sensor" => "S",
        "actuator" => "M",
        "rf" => "RF",
        "behavioral" => "X",
        _ => "A",
    }
}

fn passive_prefix(record: &CatalogRecord) -> &'static str {
    match record.model_class.as_str() {
        "Resistor" => "R",
        "Capacitor" => "C",
        "Inductor" => "L",
        "Transformer" => "T",
        "Crystal" => "Y",
        "TransmissionLine" => "TL",
        _ => "U",
    }
}

fn source_prefix(record: &CatalogRecord) -> &'static str {
    match record.model_class.as_str() {
        "Vcvs" => "E",
        "Vccs" => "G",
        "Cccs" => "F",
        "Ccvs" => "H",
        "CurrentSource" => "I",
        "Battery" => "BT",
        _ => "V",
    }
}

fn diode_prefix(record: &CatalogRecord) -> &'static str {
    match record.model_class.as_str() {
        "Optocoupler" => "U",
        _ => "D",
    }
}

fn protection_prefix(record: &CatalogRecord) -> &'static str {
    match record.name.to_ascii_lowercase().as_str() {
        name if name.contains("tvs") || name.contains("esd") => "D",
        name if name.contains("varistor") => "RV",
        _ => "F",
    }
}

fn switch_prefix(record: &CatalogRecord) -> &'static str {
    match record.model_class.as_str() {
        "Connector" => "J",
        "Relay" | "SolidStateRelay" => "K",
        _ => "SW",
    }
}

#[cfg(test)]
mod tests {
    use super::{builtin, find};
    use crate::model::SymbolKind;
    use device::catalog_records;
    use std::collections::BTreeSet;

    #[test]
    fn definition_ids_are_unique() {
        let mut ids: Vec<&str> = builtin().iter().map(|definition| definition.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), builtin().len());
    }

    #[test]
    fn pins_are_inside_definition_bounds() {
        for definition in builtin() {
            for pin in definition.pins {
                assert!(pin.offset.x >= 0);
                assert!(pin.offset.y >= 0);
                assert!(pin.offset.x <= definition.size.width);
                assert!(pin.offset.y <= definition.size.height);
            }
        }
    }

    #[test]
    fn find_returns_known_definition() {
        assert_eq!(find("resistor").expect("resistor").name, "Resistor");
        assert!(find("missing").is_none());
    }

    #[test]
    fn atmega328p_symbol_exposes_tqfp32_pins() {
        let definition = find("atmega328p").expect("mcu");
        assert_eq!(definition.pins.len(), 32);
        assert_eq!(definition.size.width, 10);
        assert_eq!(definition.size.height, 34);
        assert_eq!(definition.pins[0].name, "PD3");
        assert_eq!(definition.pins[0].number, "1");
        assert_eq!(definition.pins[16].name, "PB5");
        assert_eq!(definition.pins[28].name, "PC6/RESET");
    }

    #[test]
    fn catalog_entries_are_loaded() {
        let records = catalog_records();
        assert_eq!(records.len(), 500);
        for record in records {
            let definition = find(record.id).expect(record.id);
            assert_eq!(definition.pins.len(), usize::from(record.pins));
            assert_eq!(
                definition.category,
                crate::model::Category::from_family(record.family)
            );
        }
    }

    #[test]
    fn catalog_categories_cover_families() {
        let categories: BTreeSet<crate::model::Category> = catalog_records()
            .iter()
            .map(|record| crate::model::Category::from_family(record.family))
            .collect();
        assert_eq!(categories.len(), 16);
    }

    #[test]
    fn catalog_variants_are_unique() {
        let variants: BTreeSet<u16> = builtin()
            .iter()
            .filter_map(|definition| match definition.symbol {
                SymbolKind::Catalog { variant, .. } => Some(variant),
                _ => None,
            })
            .collect();
        assert_eq!(variants.len(), 500);
    }

    #[test]
    fn catalog_bases_exist() {
        for record in catalog_records() {
            let definition = find(record.id).expect(record.id);
            let SymbolKind::Catalog { base, .. } = definition.symbol else {
                panic!("catalog symbol missing for {}", record.id);
            };
            assert!(
                crate::symbols::asset_exists(base),
                "{} -> {}",
                record.id,
                base
            );
        }
    }
}
