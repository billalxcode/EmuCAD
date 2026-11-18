//! Embedded SVG symbol assets.

use super::svg::{self, SvgError, SvgSymbol};
use crate::model::SymbolKind;
use std::sync::OnceLock;

struct Asset {
    id: &'static str,
    text: &'static str,
}

const ASSETS: &[Asset] = &[
    Asset {
        id: "adc",
        text: include_str!("../../assets/symbols/adc.svg"),
    },
    Asset {
        id: "antenna",
        text: include_str!("../../assets/symbols/antenna.svg"),
    },
    Asset {
        id: "battery",
        text: include_str!("../../assets/symbols/battery.svg"),
    },
    Asset {
        id: "behavioral",
        text: include_str!("../../assets/symbols/behavioral.svg"),
    },
    Asset {
        id: "bjt_npn",
        text: include_str!("../../assets/symbols/bjt_npn.svg"),
    },
    Asset {
        id: "bjt_pnp",
        text: include_str!("../../assets/symbols/bjt_pnp.svg"),
    },
    Asset {
        id: "board",
        text: include_str!("../../assets/symbols/board.svg"),
    },
    Asset {
        id: "button",
        text: include_str!("../../assets/symbols/button.svg"),
    },
    Asset {
        id: "buzzer",
        text: include_str!("../../assets/symbols/buzzer.svg"),
    },
    Asset {
        id: "capacitor",
        text: include_str!("../../assets/symbols/capacitor.svg"),
    },
    Asset {
        id: "capacitor_polarized",
        text: include_str!("../../assets/symbols/capacitor_polarized.svg"),
    },
    Asset {
        id: "capacitor_trimmer",
        text: include_str!("../../assets/symbols/capacitor_trimmer.svg"),
    },
    Asset {
        id: "capacitor_variable",
        text: include_str!("../../assets/symbols/capacitor_variable.svg"),
    },
    Asset {
        id: "comparator",
        text: include_str!("../../assets/symbols/comparator.svg"),
    },
    Asset {
        id: "connector",
        text: include_str!("../../assets/symbols/connector.svg"),
    },
    Asset {
        id: "controlled_source",
        text: include_str!("../../assets/symbols/controlled_source.svg"),
    },
    Asset {
        id: "crystal",
        text: include_str!("../../assets/symbols/crystal.svg"),
    },
    Asset {
        id: "current_source",
        text: include_str!("../../assets/symbols/current_source.svg"),
    },
    Asset {
        id: "dac",
        text: include_str!("../../assets/symbols/dac.svg"),
    },
    Asset {
        id: "dcdc",
        text: include_str!("../../assets/symbols/dcdc.svg"),
    },
    Asset {
        id: "diac",
        text: include_str!("../../assets/symbols/diac.svg"),
    },
    Asset {
        id: "diode",
        text: include_str!("../../assets/symbols/diode.svg"),
    },
    Asset {
        id: "diode_array",
        text: include_str!("../../assets/symbols/diode_array.svg"),
    },
    Asset {
        id: "diode_laser",
        text: include_str!("../../assets/symbols/diode_laser.svg"),
    },
    Asset {
        id: "diode_schottky",
        text: include_str!("../../assets/symbols/diode_schottky.svg"),
    },
    Asset {
        id: "diode_tunnel",
        text: include_str!("../../assets/symbols/diode_tunnel.svg"),
    },
    Asset {
        id: "diode_varicap",
        text: include_str!("../../assets/symbols/diode_varicap.svg"),
    },
    Asset {
        id: "diode_zener",
        text: include_str!("../../assets/symbols/diode_zener.svg"),
    },
    Asset {
        id: "display",
        text: include_str!("../../assets/symbols/display.svg"),
    },
    Asset {
        id: "flip_flop",
        text: include_str!("../../assets/symbols/flip_flop.svg"),
    },
    Asset {
        id: "fuse",
        text: include_str!("../../assets/symbols/fuse.svg"),
    },
    Asset {
        id: "gnd",
        text: include_str!("../../assets/symbols/gnd.svg"),
    },
    Asset {
        id: "ic",
        text: include_str!("../../assets/symbols/ic.svg"),
    },
    Asset {
        id: "igbt",
        text: include_str!("../../assets/symbols/igbt.svg"),
    },
    Asset {
        id: "inductor",
        text: include_str!("../../assets/symbols/inductor.svg"),
    },
    Asset {
        id: "inductor_ferrite",
        text: include_str!("../../assets/symbols/inductor_ferrite.svg"),
    },
    Asset {
        id: "inductor_variable",
        text: include_str!("../../assets/symbols/inductor_variable.svg"),
    },
    Asset {
        id: "jfet_n",
        text: include_str!("../../assets/symbols/jfet_n.svg"),
    },
    Asset {
        id: "jfet_p",
        text: include_str!("../../assets/symbols/jfet_p.svg"),
    },
    Asset {
        id: "lamp",
        text: include_str!("../../assets/symbols/lamp.svg"),
    },
    Asset {
        id: "led",
        text: include_str!("../../assets/symbols/led.svg"),
    },
    Asset {
        id: "logic_and",
        text: include_str!("../../assets/symbols/logic_and.svg"),
    },
    Asset {
        id: "logic_buffer",
        text: include_str!("../../assets/symbols/logic_buffer.svg"),
    },
    Asset {
        id: "logic_nand",
        text: include_str!("../../assets/symbols/logic_nand.svg"),
    },
    Asset {
        id: "logic_nor",
        text: include_str!("../../assets/symbols/logic_nor.svg"),
    },
    Asset {
        id: "logic_not",
        text: include_str!("../../assets/symbols/logic_not.svg"),
    },
    Asset {
        id: "logic_or",
        text: include_str!("../../assets/symbols/logic_or.svg"),
    },
    Asset {
        id: "logic_schmitt",
        text: include_str!("../../assets/symbols/logic_schmitt.svg"),
    },
    Asset {
        id: "logic_xnor",
        text: include_str!("../../assets/symbols/logic_xnor.svg"),
    },
    Asset {
        id: "logic_xor",
        text: include_str!("../../assets/symbols/logic_xor.svg"),
    },
    Asset {
        id: "mcu",
        text: include_str!("../../assets/symbols/mcu.svg"),
    },
    Asset {
        id: "memristor",
        text: include_str!("../../assets/symbols/memristor.svg"),
    },
    Asset {
        id: "mosfet_n",
        text: include_str!("../../assets/symbols/mosfet_n.svg"),
    },
    Asset {
        id: "mosfet_p",
        text: include_str!("../../assets/symbols/mosfet_p.svg"),
    },
    Asset {
        id: "motor",
        text: include_str!("../../assets/symbols/motor.svg"),
    },
    Asset {
        id: "opamp",
        text: include_str!("../../assets/symbols/opamp.svg"),
    },
    Asset {
        id: "optocoupler",
        text: include_str!("../../assets/symbols/optocoupler.svg"),
    },
    Asset {
        id: "photodiode",
        text: include_str!("../../assets/symbols/photodiode.svg"),
    },
    Asset {
        id: "photoresistor",
        text: include_str!("../../assets/symbols/photoresistor.svg"),
    },
    Asset {
        id: "potentiometer",
        text: include_str!("../../assets/symbols/potentiometer.svg"),
    },
    Asset {
        id: "regulator",
        text: include_str!("../../assets/symbols/regulator.svg"),
    },
    Asset {
        id: "relay",
        text: include_str!("../../assets/symbols/relay.svg"),
    },
    Asset {
        id: "relay_spdt",
        text: include_str!("../../assets/symbols/relay_spdt.svg"),
    },
    Asset {
        id: "resistor",
        text: include_str!("../../assets/symbols/resistor.svg"),
    },
    Asset {
        id: "rf",
        text: include_str!("../../assets/symbols/rf.svg"),
    },
    Asset {
        id: "scr",
        text: include_str!("../../assets/symbols/scr.svg"),
    },
    Asset {
        id: "sensor",
        text: include_str!("../../assets/symbols/sensor.svg"),
    },
    Asset {
        id: "speaker",
        text: include_str!("../../assets/symbols/speaker.svg"),
    },
    Asset {
        id: "square_source",
        text: include_str!("../../assets/symbols/square_source.svg"),
    },
    Asset {
        id: "switch",
        text: include_str!("../../assets/symbols/switch.svg"),
    },
    Asset {
        id: "switch_spdt",
        text: include_str!("../../assets/symbols/switch_spdt.svg"),
    },
    Asset {
        id: "thermistor",
        text: include_str!("../../assets/symbols/thermistor.svg"),
    },
    Asset {
        id: "transformer",
        text: include_str!("../../assets/symbols/transformer.svg"),
    },
    Asset {
        id: "transformer_center",
        text: include_str!("../../assets/symbols/transformer_center.svg"),
    },
    Asset {
        id: "transmission_line",
        text: include_str!("../../assets/symbols/transmission_line.svg"),
    },
    Asset {
        id: "triac",
        text: include_str!("../../assets/symbols/triac.svg"),
    },
    Asset {
        id: "varistor",
        text: include_str!("../../assets/symbols/varistor.svg"),
    },
    Asset {
        id: "vcc",
        text: include_str!("../../assets/symbols/vcc.svg"),
    },
    Asset {
        id: "voltage_source",
        text: include_str!("../../assets/symbols/voltage_source.svg"),
    },
];

struct ParsedAsset {
    id: &'static str,
    result: Result<SvgSymbol, SvgError>,
}

static PARSED: OnceLock<Vec<ParsedAsset>> = OnceLock::new();

/// Returns the parsed symbol for a library symbol kind.
pub(crate) fn symbol(kind: SymbolKind) -> Option<&'static SvgSymbol> {
    let id = asset_id(kind);
    parsed()
        .iter()
        .find(|asset| asset.id == id)
        .and_then(|asset| asset.result.as_ref().ok())
}

/// Returns true when an asset ID is registered.
#[cfg(test)]
pub(crate) fn contains(id: &str) -> bool {
    ASSETS.iter().any(|asset| asset.id == id)
}

/// Returns the embedded asset ID for a library symbol kind.
pub(crate) const fn asset_id(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Resistor => "resistor",
        SymbolKind::Led => "led",
        SymbolKind::Button => "button",
        SymbolKind::Vcc => "vcc",
        SymbolKind::Gnd => "gnd",
        SymbolKind::Mcu => "mcu",
        SymbolKind::Catalog { base, .. } => base,
    }
}

fn parsed() -> &'static [ParsedAsset] {
    PARSED.get_or_init(|| {
        ASSETS
            .iter()
            .map(|asset| ParsedAsset {
                id: asset.id,
                result: svg::parse(asset.text),
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::{ASSETS, asset_id, contains, parsed, symbol};
    use crate::model::SymbolKind;

    const KINDS: [SymbolKind; 6] = [
        SymbolKind::Resistor,
        SymbolKind::Led,
        SymbolKind::Button,
        SymbolKind::Vcc,
        SymbolKind::Gnd,
        SymbolKind::Mcu,
    ];

    #[test]
    fn every_symbol_kind_has_a_parsed_asset() {
        for kind in KINDS {
            assert!(symbol(kind).is_some(), "missing {}", asset_id(kind));
        }
    }

    #[test]
    fn every_asset_parses_and_has_bounds() {
        for asset in parsed() {
            let symbol = asset.result.as_ref().expect(asset.id);
            assert!(!symbol.shapes.is_empty(), "empty {}", asset.id);
            assert!(symbol.bounds.width() > 0.0, "width {}", asset.id);
            assert!(symbol.bounds.height() > 0.0, "height {}", asset.id);
        }
    }

    #[test]
    fn asset_ids_are_unique() {
        let mut ids: Vec<&str> = ASSETS.iter().map(|asset| asset.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ASSETS.len());
    }

    #[test]
    fn asset_text_is_ascii() {
        for asset in ASSETS {
            assert!(asset.text.is_ascii(), "non-ASCII asset {}", asset.id);
        }
    }

    #[test]
    fn known_base_ids_exist() {
        assert!(contains("resistor"));
        assert!(contains("ic"));
        assert!(contains("logic_and"));
        assert!(!contains("missing"));
    }
}
