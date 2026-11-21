//! SVG symbol assets and rendering.

pub(crate) mod assets;
pub(crate) mod catalog;
mod draw;
mod path;
mod svg;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use assets::contains as asset_exists;
pub(crate) use catalog::base_for;
pub(crate) use draw::{draw_svg, draw_symbol};
pub(crate) use svg::{SvgSymbol, parse as parse_svg};
