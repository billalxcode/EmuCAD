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
        id: "resistor",
        text: include_str!("../../assets/symbols/resistor.svg"),
    },
    Asset {
        id: "led",
        text: include_str!("../../assets/symbols/led.svg"),
    },
    Asset {
        id: "button",
        text: include_str!("../../assets/symbols/button.svg"),
    },
    Asset {
        id: "vcc",
        text: include_str!("../../assets/symbols/vcc.svg"),
    },
    Asset {
        id: "gnd",
        text: include_str!("../../assets/symbols/gnd.svg"),
    },
    Asset {
        id: "mcu",
        text: include_str!("../../assets/symbols/mcu.svg"),
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

/// Returns the embedded asset ID for a library symbol kind.
pub(crate) const fn asset_id(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Resistor => "resistor",
        SymbolKind::Led => "led",
        SymbolKind::Button => "button",
        SymbolKind::Vcc => "vcc",
        SymbolKind::Gnd => "gnd",
        SymbolKind::Mcu => "mcu",
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
    use super::{ASSETS, asset_id, parsed, symbol};
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
}
