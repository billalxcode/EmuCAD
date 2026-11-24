//! Paints parsed SVG symbols into egui rectangles.

use super::assets;
use super::svg::{Bounds, Point, SvgShape, SvgSymbol};
use crate::model::SymbolKind;
use eframe::egui;

const MIN_STROKE: f32 = 1.25;
const MAX_STROKE: f32 = 2.0;
const CODE_MARGIN: f32 = 5.0;
const CODE_STEP: f32 = 4.0;
const CODE_RADIUS: f32 = 1.1;

/// Draws one library symbol into a screen rectangle.
pub(crate) fn draw_symbol(
    painter: &egui::Painter,
    body: egui::Rect,
    kind: SymbolKind,
    color: egui::Color32,
) {
    let Some(symbol) = assets::symbol(kind) else {
        return;
    };
    draw_svg(painter, body, symbol, color);
    if let SymbolKind::Catalog { variant, .. } = kind {
        draw_variant_code(painter, body, variant, color);
    }
}

pub(crate) fn draw_svg(
    painter: &egui::Painter,
    body: egui::Rect,
    symbol: &SvgSymbol,
    color: egui::Color32,
) {
    let Some(fit) = Fit::new(symbol.view_box, body) else {
        return;
    };
    for shape in &symbol.shapes {
        draw_shape(painter, shape, fit, color);
    }
}

fn draw_variant_code(
    painter: &egui::Painter,
    body: egui::Rect,
    variant: u16,
    color: egui::Color32,
) {
    if body.width() < 32.0 || body.height() < 24.0 {
        return;
    }
    let origin = body.left_top() + egui::vec2(CODE_MARGIN, CODE_MARGIN);
    for bit in 0..9 {
        draw_code_dot(painter, origin, bit, variant, color);
    }
}

fn draw_code_dot(
    painter: &egui::Painter,
    origin: egui::Pos2,
    bit: usize,
    variant: u16,
    color: egui::Color32,
) {
    let center = origin + egui::vec2((bit % 3) as f32 * CODE_STEP, (bit / 3) as f32 * CODE_STEP);
    if variant & (1 << bit) != 0 {
        painter.circle_filled(center, CODE_RADIUS, with_alpha(color, 0.55));
        return;
    }
    let stroke = egui::Stroke::new(0.6, with_alpha(color, 0.22));
    painter.circle_stroke(center, CODE_RADIUS, stroke);
}

#[derive(Clone, Copy)]
struct Fit {
    scale: egui::Vec2,
    offset: egui::Vec2,
}

impl Fit {
    fn new(view_box: Bounds, body: egui::Rect) -> Option<Self> {
        if view_box.width() <= 0.0 || view_box.height() <= 0.0 {
            return None;
        }
        let scale = egui::vec2(
            body.width() / view_box.width(),
            body.height() / view_box.height(),
        );
        if !scale.x.is_finite() || !scale.y.is_finite() {
            return None;
        }
        let center = view_box.center();
        let offset = body.center().to_vec2() - egui::vec2(center.x * scale.x, center.y * scale.y);
        Some(Self { scale, offset })
    }

    fn point(self, point: Point) -> egui::Pos2 {
        egui::pos2(
            point.x * self.scale.x + self.offset.x,
            point.y * self.scale.y + self.offset.y,
        )
    }

    fn stroke_scale(self) -> f32 {
        (self.scale.x + self.scale.y) * 0.5
    }
}

fn draw_shape(painter: &egui::Painter, shape: &SvgShape, fit: Fit, color: egui::Color32) {
    if shape.points.len() < 2 {
        return;
    }
    let points: Vec<egui::Pos2> = shape.points.iter().map(|point| fit.point(*point)).collect();
    if let Some(alpha) = shape.fill {
        paint_fill(painter, &points, color, alpha);
    }
    if let Some(alpha) = shape.stroke {
        let width = stroke_width(shape.width, fit.stroke_scale());
        paint_stroke(painter, &points, shape.closed, color, alpha, width);
    }
}

fn stroke_width(source_width: f32, scale: f32) -> f32 {
    (source_width * scale).clamp(MIN_STROKE, MAX_STROKE)
}

fn paint_fill(painter: &egui::Painter, points: &[egui::Pos2], color: egui::Color32, alpha: f32) {
    if points.len() < 3 {
        return;
    }
    let fill = with_alpha(color, alpha);
    painter.add(egui::epaint::PathShape::convex_polygon(
        points.to_vec(),
        fill,
        egui::Stroke::NONE,
    ));
}

fn paint_stroke(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    closed: bool,
    color: egui::Color32,
    alpha: f32,
    width: f32,
) {
    let stroke = egui::Stroke::new(width, with_alpha(color, alpha));
    let shape = if closed {
        egui::epaint::PathShape::closed_line(points.to_vec(), stroke)
    } else {
        egui::epaint::PathShape::line(points.to_vec(), stroke)
    };
    painter.add(shape);
}

fn with_alpha(color: egui::Color32, alpha: f32) -> egui::Color32 {
    let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

#[cfg(test)]
mod tests {
    use super::{Fit, draw_symbol, with_alpha};
    use crate::model::SymbolKind;
    use crate::symbols::svg::Point;
    use eframe::egui;

    const KINDS: [SymbolKind; 6] = [
        SymbolKind::Resistor,
        SymbolKind::Led,
        SymbolKind::Button,
        SymbolKind::Vcc,
        SymbolKind::Gnd,
        SymbolKind::Mcu,
    ];

    #[test]
    fn every_symbol_draws_shapes() {
        let context = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 240.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            let body = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(80.0, 40.0));
            for kind in KINDS {
                draw_symbol(ui.painter(), body, kind, egui::Color32::WHITE);
            }
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
    }

    fn catalog_input() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 240.0),
            )),
            ..Default::default()
        }
    }

    fn draw_catalog_symbols(
        context: &egui::Context,
        definitions: &[crate::model::ComponentDefinition],
    ) -> usize {
        let mut count = 0;
        let mut output = context.run_ui(catalog_input(), |ui| {
            let body = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(80.0, 40.0));
            for definition in definitions {
                if matches!(definition.symbol, SymbolKind::Catalog { .. }) {
                    draw_symbol(ui.painter(), body, definition.symbol, egui::Color32::WHITE);
                    count += 1;
                }
            }
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
        count
    }

    #[test]
    fn catalog_symbols_draw_shapes() {
        let definitions = crate::library::builtin();
        let count = draw_catalog_symbols(&egui::Context::default(), definitions);
        assert_eq!(count, 500);
    }

    #[test]
    fn fit_maps_view_box_to_body() {
        let text = r##"<svg viewBox="0 0 150 150"><path fill="none" stroke="#000" d="M0 0H150V150H0Z"/></svg>"##;
        let symbol = crate::symbols::parse_svg(text).expect("symbol");
        let body = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(60.0, 40.0));
        let fit = Fit::new(symbol.view_box, body).expect("fit");
        let center = fit.point(Point::new(75.0, 75.0));
        let top_left = fit.point(Point::new(0.0, 0.0));
        assert!((center - body.center()).length() < 0.01);
        assert!((top_left - body.left_top()).length() < 0.01);
    }

    #[test]
    fn stroke_width_is_bounded() {
        assert_eq!(super::stroke_width(5.0, 4.0), super::MAX_STROKE);
        assert_eq!(super::stroke_width(5.0, 0.1), super::MIN_STROKE);
        assert!((super::stroke_width(5.0, 0.3) - 1.5).abs() < 0.01);
    }

    #[test]
    fn alpha_is_clamped() {
        let color = egui::Color32::from_rgb(10, 20, 30);
        assert_eq!(with_alpha(color, 2.0).a(), 255);
        assert_eq!(with_alpha(color, -0.5).a(), 0);
    }
}
