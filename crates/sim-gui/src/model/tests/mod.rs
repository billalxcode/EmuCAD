use super::GridRect;
use super::{
    ComponentDefinition, Document, GridPos, GridSize, Mirror, NetClass, PinDefinition, Rotation,
    SymbolKind, Wire, WireId, body_size, world_pin,
};
use crate::library;

const PINS: &[PinDefinition] = &[
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

const RESISTOR: ComponentDefinition = ComponentDefinition {
    id: "resistor",
    name: "Resistor",
    category: super::Category::Passive,
    reference_prefix: "R",
    default_value: "220",
    size: GridSize::new(4, 2),
    pins: PINS,
    symbol: SymbolKind::Resistor,
};

const LIBRARY: &[ComponentDefinition] = &[RESISTOR];

mod benchmark;
mod editing;
mod geometry;
mod index;
mod segments;
mod wires;

fn test_wire(id: u32, start: GridPos, end: GridPos, class: NetClass) -> Wire {
    Wire {
        id: WireId(id),
        points: vec![start, end],
        class,
        color_override: None,
        label: None,
    }
}

fn segment_document() -> (Document, super::WireId) {
    let library = library::builtin();
    let resistor = library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let points = vec![
        GridPos::new(4, 1),
        GridPos::new(4, 0),
        GridPos::new(6, 0),
        GridPos::new(6, 1),
    ];
    let wire = document
        .add_wire(library, points, NetClass::Digital)
        .expect("wire");
    (document, wire)
}
