use super::svg::{self, Point, SvgError};

#[test]
fn parses_a_simple_path() {
    let text = r##"<svg viewBox="0 0 10 10"><path fill="none" stroke="#000" stroke-width="1" d="M0 0L10 10"/></svg>"##;
    let symbol = svg::parse(text).expect("symbol");
    assert_eq!(symbol.shapes.len(), 1);
    assert_eq!(symbol.shapes[0].points.len(), 2);
    assert_eq!(symbol.bounds.width(), 10.0);
    assert_eq!(symbol.bounds.height(), 10.0);
}

#[test]
fn group_style_is_inherited() {
    let text = r##"<svg><g fill="none" stroke="#000" stroke-width="2"><line x1="0" y1="0" x2="10" y2="10"/></g></svg>"##;
    let symbol = svg::parse(text).expect("symbol");
    let shape = &symbol.shapes[0];
    assert_eq!(shape.fill, None);
    assert_eq!(shape.stroke, Some(1.0));
    assert_eq!(shape.width, 2.0);
}

#[test]
fn transform_moves_geometry() {
    let text = r##"<svg><g transform="translate(10 20)"><rect x="0" y="0" width="5" height="5"/></g></svg>"##;
    let symbol = svg::parse(text).expect("symbol");
    assert_eq!(symbol.bounds.center(), Point::new(12.5, 22.5));
}

#[test]
fn arcs_are_supported() {
    let text = r##"<svg><path d="M0 0A1 1 0 0 0 2 2"/></svg>"##;
    let symbol = svg::parse(text).expect("arc symbol");
    assert!(!symbol.shapes.is_empty());
}

#[test]
fn text_is_ignored() {
    let text = r##"<svg><text x="0" y="0">R1</text><path d="M0 0L1 1"/></svg>"##;
    let symbol = svg::parse(text).expect("symbol");
    assert_eq!(symbol.shapes.len(), 1);
}

#[test]
fn empty_documents_are_rejected() {
    assert_eq!(svg::parse(""), Err(SvgError::Empty));
}

#[test]
fn circles_use_enough_segments() {
    let text = r##"<svg><circle cx="5" cy="5" r="3"/></svg>"##;
    let symbol = svg::parse(text).expect("symbol");
    assert!(symbol.shapes[0].points.len() >= 64);
}
