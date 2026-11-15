//! SVG symbol assets and rendering.

mod assets;
mod draw;
mod path;
mod svg;
#[cfg(test)]
mod tests;

pub(crate) use draw::draw_symbol;
