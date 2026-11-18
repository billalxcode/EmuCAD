//! Schematic document model, grid geometry, and component transforms.

use crate::spatial::SegmentIndex;
use std::collections::{BTreeMap, BTreeSet};

/// Integer position on the schematic grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    /// Creates a grid position.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Adds another grid offset.
    #[must_use]
    pub const fn offset(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Subtracts another grid offset.
    #[must_use]
    pub const fn subtract(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Size in grid cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridSize {
    pub width: i32,
    pub height: i32,
}

impl GridSize {
    /// Creates a grid size.
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Returns the size with width and height swapped.
    #[must_use]
    pub const fn swapped(self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }
}

/// Axis-aligned rectangle on the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridRect {
    pub min: GridPos,
    pub size: GridSize,
}

impl GridRect {
    /// Creates a rectangle from its top-left position and size.
    #[must_use]
    pub const fn new(min: GridPos, size: GridSize) -> Self {
        Self { min, size }
    }

    /// Returns the exclusive right edge.
    #[must_use]
    pub const fn max_x(self) -> i32 {
        self.min.x + self.size.width
    }

    /// Returns the exclusive bottom edge.
    #[must_use]
    pub const fn max_y(self) -> i32 {
        self.min.y + self.size.height
    }

    /// Returns true when a grid point lies inside or on the rectangle.
    #[must_use]
    pub const fn contains(self, point: GridPos) -> bool {
        point.x >= self.min.x
            && point.x <= self.max_x()
            && point.y >= self.min.y
            && point.y <= self.max_y()
    }

    /// Returns true when a grid point lies strictly inside the rectangle.
    #[must_use]
    pub const fn strictly_contains(self, point: GridPos) -> bool {
        point.x > self.min.x
            && point.x < self.max_x()
            && point.y > self.min.y
            && point.y < self.max_y()
    }

    /// Returns true when two rectangles overlap in their interiors.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.min.x < other.max_x()
            && other.min.x < self.max_x()
            && self.min.y < other.max_y()
            && other.min.y < self.max_y()
    }

    /// Builds an inclusive rectangle from two corners.
    #[must_use]
    pub fn from_corners(first: GridPos, second: GridPos) -> Self {
        let min = GridPos::new(first.x.min(second.x), first.y.min(second.y));
        let width = (first.x - second.x).abs().max(1);
        let height = (first.y - second.y).abs().max(1);
        Self::new(min, GridSize::new(width, height))
    }
}

/// Component rotation in quarter turns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    /// Returns the next clockwise rotation.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Deg0 => "0",
            Self::Deg90 => "90",
            Self::Deg180 => "180",
            Self::Deg270 => "270",
        }
    }

    fn apply(self, delta: GridPos) -> GridPos {
        match self {
            Self::Deg0 => delta,
            Self::Deg90 => GridPos::new(delta.y, -delta.x),
            Self::Deg180 => GridPos::new(-delta.x, -delta.y),
            Self::Deg270 => GridPos::new(-delta.y, delta.x),
        }
    }
}

/// Component mirroring across an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mirror {
    #[default]
    None,
    Horizontal,
    Vertical,
}

impl Mirror {
    /// Returns the next mirror mode.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::Horizontal,
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::None,
        }
    }

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    fn apply(self, delta: GridPos) -> GridPos {
        match self {
            Self::None => delta,
            Self::Horizontal => GridPos::new(-delta.x, delta.y),
            Self::Vertical => GridPos::new(delta.x, -delta.y),
        }
    }
}

/// One pin on a component definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinDefinition {
    pub name: &'static str,
    pub number: &'static str,
    pub offset: GridPos,
}

/// Symbol family used by the canvas renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Resistor,
    Led,
    Button,
    Vcc,
    Gnd,
    Mcu,
    Catalog { base: &'static str, variant: u16 },
}

/// Library category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Board,
    Passive,
    Source,
    Diode,
    Transistor,
    Protection,
    Switch,
    Analog,
    Logic,
    Memory,
    Mcu,
    Power,
    Sensor,
    Actuator,
    Rf,
    Behavioral,
}

impl Category {
    pub const ALL: [Self; 16] = [
        Self::Board,
        Self::Passive,
        Self::Source,
        Self::Diode,
        Self::Transistor,
        Self::Protection,
        Self::Switch,
        Self::Analog,
        Self::Logic,
        Self::Memory,
        Self::Mcu,
        Self::Power,
        Self::Sensor,
        Self::Actuator,
        Self::Rf,
        Self::Behavioral,
    ];

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Passive => "Passive",
            Self::Source => "Source",
            Self::Diode => "Diode",
            Self::Transistor => "Transistor",
            Self::Protection => "Protection",
            Self::Switch => "Switch",
            Self::Analog => "Analog",
            Self::Logic => "Logic",
            Self::Memory => "Memory",
            Self::Mcu => "MCU",
            Self::Power => "Power",
            Self::Sensor => "Sensor",
            Self::Actuator => "Actuator",
            Self::Rf => "RF",
            Self::Behavioral => "Behavioral",
        }
    }

    /// Maps a catalog family name to a library category.
    #[must_use]
    pub fn from_family(family: &str) -> Self {
        match family {
            "passive" => Self::Passive,
            "source" => Self::Source,
            "diode" => Self::Diode,
            "transistor" => Self::Transistor,
            "protection" => Self::Protection,
            "switch" => Self::Switch,
            "analog" => Self::Analog,
            "logic" => Self::Logic,
            "memory" => Self::Memory,
            "mcu" => Self::Mcu,
            "power" => Self::Power,
            "sensor" => Self::Sensor,
            "actuator" => Self::Actuator,
            "rf" => Self::Rf,
            "behavioral" => Self::Behavioral,
            _ => Self::Board,
        }
    }
}

/// Static component definition.
#[derive(Clone, Copy, Debug)]
pub struct ComponentDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub reference_prefix: &'static str,
    pub default_value: &'static str,
    pub size: GridSize,
    pub pins: &'static [PinDefinition],
    pub symbol: SymbolKind,
}

/// Stable component instance identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(pub u32);

/// Stable wire identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireId(pub u32);

/// Net class used for wire coloring and semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NetClass {
    Power,
    Ground,
    #[default]
    Digital,
    Analog,
    Clock,
    Uart,
    Spi,
    I2c,
    Pwm,
    Reset,
    User,
}

impl NetClass {
    pub const ALL: [Self; 11] = [
        Self::Power,
        Self::Ground,
        Self::Digital,
        Self::Analog,
        Self::Clock,
        Self::Uart,
        Self::Spi,
        Self::I2c,
        Self::Pwm,
        Self::Reset,
        Self::User,
    ];

    /// Returns a human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Power => "Power",
            Self::Ground => "Ground",
            Self::Digital => "Digital",
            Self::Analog => "Analog",
            Self::Clock => "Clock",
            Self::Uart => "UART",
            Self::Spi => "SPI",
            Self::I2c => "I2C",
            Self::Pwm => "PWM",
            Self::Reset => "Reset",
            Self::User => "User",
        }
    }
}

/// One placed wire polyline.
#[derive(Clone, Debug)]
pub struct Wire {
    pub id: WireId,
    pub points: Vec<GridPos>,
    pub class: NetClass,
    pub color_override: Option<[u8; 3]>,
    pub label: Option<String>,
}

fn new_wire(id: WireId, points: Vec<GridPos>, class: NetClass) -> Wire {
    Wire {
        id,
        points,
        class,
        color_override: None,
        label: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct GridEdge {
    first: GridPos,
    second: GridPos,
}

impl GridEdge {
    fn new(first: GridPos, second: GridPos) -> Self {
        if first <= second {
            Self { first, second }
        } else {
            Self {
                first: second,
                second: first,
            }
        }
    }
}

/// One placed component.
#[derive(Clone, Debug)]
pub struct ComponentInstance {
    pub id: ComponentId,
    pub definition: &'static str,
    pub position: GridPos,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub label: String,
    pub value: String,
    pub parameters: BTreeMap<String, f64>,
}

impl ComponentInstance {
    /// Returns the placed body rectangle.
    #[must_use]
    pub fn bounds(&self, definition: &ComponentDefinition) -> GridRect {
        GridRect::new(self.position, body_size(definition, self.rotation))
    }
}

/// Schematic document containing components and wires.
#[derive(Clone, Default)]
pub struct Document {
    components: Vec<ComponentInstance>,
    wires: Vec<Wire>,
    wire_index: SegmentIndex,
    next_id: u32,
    next_wire_id: u32,
}

mod components;
mod document;
mod geometry;
#[cfg(test)]
mod tests;

pub(super) use geometry::*;
