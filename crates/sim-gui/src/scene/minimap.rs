use super::*;

#[derive(Clone, Copy)]
pub(crate) struct MinimapMap {
    pub(crate) origin: egui::Pos2,
    pub(crate) scale: f32,
}

impl MinimapMap {
    pub(crate) fn point(self, point: GridPos) -> egui::Pos2 {
        self.origin + egui::vec2(point.x as f32, point.y as f32) * self.scale
    }
}

pub(crate) fn draw_minimap(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    if rect.width() < 360.0 || rect.height() < 260.0 {
        return;
    }
    let area = minimap_area(rect);
    paint_minimap_frame(painter, area);
    let Some(map) = minimap_for(area, scene) else {
        return;
    };
    draw_minimap_wires(painter, map, scene.document);
    draw_minimap_components(painter, map, scene.document, scene.library);
    draw_minimap_viewport(painter, map, rect, camera);
}

pub(crate) fn minimap_area(rect: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(rect.right() - 172.0, rect.bottom() - 122.0),
        egui::vec2(160.0, 110.0),
    )
}

pub(crate) fn paint_minimap_frame(painter: &egui::Painter, area: egui::Rect) {
    painter.rect_filled(area, 4.0, egui::Color32::from_black_alpha(205));
    painter.rect_stroke(
        area,
        4.0,
        egui::Stroke::new(1.0, JUNCTION_COLOR),
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn minimap_for(area: egui::Rect, scene: &Scene<'_>) -> Option<MinimapMap> {
    let (min, max) = scene.document.bounds(scene.library)?;
    minimap_map(area, min, max)
}

pub(crate) fn minimap_map(area: egui::Rect, min: GridPos, max: GridPos) -> Option<MinimapMap> {
    let inner = area.shrink(6.0);
    let width = (max.x - min.x).max(1) as f32;
    let height = (max.y - min.y).max(1) as f32;
    let scale = (inner.width() / width).min(inner.height() / height);
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let center = egui::vec2((min.x + max.x) as f32 * 0.5, (min.y + max.y) as f32 * 0.5);
    Some(MinimapMap {
        origin: inner.center() - center * scale,
        scale,
    })
}

pub(crate) fn draw_minimap_wires(painter: &egui::Painter, map: MinimapMap, document: &Document) {
    for wire in document.wires() {
        let color = wire_color(wire).gamma_multiply(0.8);
        for segment in wire.points.windows(2) {
            painter.line_segment(
                [map.point(segment[0]), map.point(segment[1])],
                egui::Stroke::new(1.0, color),
            );
        }
    }
}

pub(crate) fn draw_minimap_components(
    painter: &egui::Painter,
    map: MinimapMap,
    document: &Document,
    definitions: &[ComponentDefinition],
) {
    for instance in document.components() {
        let Some(definition) = definitions
            .iter()
            .find(|definition| definition.id == instance.definition)
        else {
            continue;
        };
        painter.rect_filled(
            minimap_component_rect(map, instance, definition),
            1.0,
            egui::Color32::from_rgb(120, 130, 150),
        );
    }
}

pub(crate) fn minimap_component_rect(
    map: MinimapMap,
    instance: &crate::model::ComponentInstance,
    definition: &ComponentDefinition,
) -> egui::Rect {
    let bounds = instance.bounds(definition);
    let min = map.point(bounds.min);
    let max = map.point(GridPos::new(bounds.max_x(), bounds.max_y()));
    egui::Rect::from_min_max(min, max)
}

pub(crate) fn draw_minimap_viewport(
    painter: &egui::Painter,
    map: MinimapMap,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
) {
    let min = camera.screen_to_world(rect, rect.min);
    let max = camera.screen_to_world(rect, rect.max);
    let min = map.point(GridPos::new(min.x.round() as i32, min.y.round() as i32));
    let max = map.point(GridPos::new(max.x.round() as i32, max.y.round() as i32));
    painter.rect_stroke(
        egui::Rect::from_two_pos(min, max),
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 220, 120)),
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn screen_box(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> egui::Rect {
    let first = screen_pos(rect, camera, start);
    let second = screen_pos(rect, camera, end);
    egui::Rect::from_two_pos(first, second)
}
