//! Paints parsed SVG symbols into egui rectangles.

use super::assets;
use super::svg::{Bounds, Point, SvgShape};
use crate::model::SymbolKind;
use eframe::egui;

const MIN_STROKE: f32 = 1.25;

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
    let Some(fit) = Fit::new(symbol.bounds, body) else {
        return;
    };
    for shape in &symbol.shapes {
        draw_shape(painter, shape, fit, color);
    }
}

#[derive(Clone, Copy)]
struct Fit {
    scale: f32,
    offset: egui::Vec2,
}

impl Fit {
    fn new(bounds: Bounds, body: egui::Rect) -> Option<Self> {
        let width = body.width() - 2.0;
        let height = body.height() - 2.0;
        if width <= 0.0 || height <= 0.0 || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
            return None;
        }
        let scale = (width / bounds.width()).min(height / bounds.height());
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let center = bounds.center();
        let offset = body.center().to_vec2() - egui::vec2(center.x * scale, center.y * scale);
        Some(Self { scale, offset })
    }

    fn point(self, point: Point) -> egui::Pos2 {
        egui::pos2(
            point.x * self.scale + self.offset.x,
            point.y * self.scale + self.offset.y,
        )
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
        let width = (shape.width * fit.scale).max(MIN_STROKE);
        paint_stroke(painter, &points, shape.closed, color, alpha, width);
    }
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
    use super::{draw_symbol, with_alpha};
    use crate::model::SymbolKind;
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

    #[test]
    fn alpha_is_clamped() {
        let color = egui::Color32::from_rgb(10, 20, 30);
        assert_eq!(with_alpha(color, 2.0).a(), 255);
        assert_eq!(with_alpha(color, -0.5).a(), 0);
    }
}
