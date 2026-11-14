use super::*;

pub(crate) fn draw_wire_handles(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    for wire in scene.document.wires() {
        if !scene.selection.contains_wire(wire.id) {
            continue;
        }
        for segment in wire.points.windows(2) {
            let midpoint = segment_midpoint(segment[0], segment[1]);
            let screen = camera.world_to_screen(rect, midpoint);
            draw_handle(painter, screen);
        }
    }
}

pub(crate) fn segment_midpoint(first: GridPos, second: GridPos) -> egui::Pos2 {
    egui::pos2(
        (first.x + second.x) as f32 * 0.5,
        (first.y + second.y) as f32 * 0.5,
    )
}

pub(crate) fn draw_handle(painter: &egui::Painter, screen: egui::Pos2) {
    let handle = egui::Rect::from_center_size(screen, egui::vec2(8.0, 8.0));
    painter.rect_filled(handle, egui::CornerRadius::ZERO, SELECTED_STROKE);
    painter.rect_stroke(
        handle,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(1.0, JUNCTION_COLOR),
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn draw_wires(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selection: &Selection,
    hovered: Option<WireId>,
    flow_phase: Option<f32>,
) {
    draw_wire_lines(painter, rect, camera, document, selection, flow_phase);
    draw_wire_labels(painter, rect, camera, document, selection, hovered);
}

pub(crate) fn draw_wire_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selection: &Selection,
    flow_phase: Option<f32>,
) {
    for wire in document.wires() {
        if !wire_visible(rect, camera, wire) {
            continue;
        }
        let selected = selection.contains_wire(wire.id);
        draw_wire(painter, rect, camera, wire, selected, flow_phase);
    }
}

pub(crate) fn draw_wire_labels(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
    selection: &Selection,
    hovered: Option<WireId>,
) {
    for wire in document.wires() {
        if !wire_visible(rect, camera, wire) {
            continue;
        }
        let selected = selection.contains_wire(wire.id);
        if show_wire_label(selected, hovered == Some(wire.id), camera.zoom) {
            draw_net_label(painter, rect, camera, wire);
        }
    }
}

pub(crate) fn show_wire_label(selected: bool, hovered: bool, zoom: f32) -> bool {
    selected || hovered || zoom >= 1.5
}

pub(crate) fn draw_net_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) {
    let Some(midpoint) = wire_midpoint(wire) else {
        return;
    };
    let pos = camera.world_to_screen(rect, midpoint) + egui::vec2(4.0, -16.0);
    let color = wire_color(wire);
    let galley = painter.layout_no_wrap(
        wire_label_text(wire).to_string(),
        egui::FontId::monospace(11.0),
        color,
    );
    let background = egui::Rect::from_min_size(pos, galley.size()).expand(3.0);
    painter.rect_filled(background, 3.0, egui::Color32::from_black_alpha(190));
    painter.galley(pos, galley, color);
}

pub(crate) fn wire_midpoint(wire: &crate::model::Wire) -> Option<egui::Pos2> {
    let index = wire.points.len().saturating_sub(1) / 2;
    let segment = wire.points.windows(2).nth(index)?;
    Some(segment_midpoint(segment[0], segment[1]))
}

pub(crate) fn draw_wire(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
    selected: bool,
    flow_phase: Option<f32>,
) {
    if selected {
        let halo = SELECTED_STROKE.gamma_multiply(0.6);
        draw_polyline(painter, rect, camera, &wire.points, halo, 6.0);
        draw_wire_ends(painter, rect, camera, wire);
    }
    let style = wire_style(wire_color(wire), wire.class, flow_phase);
    draw_patterned_polyline(painter, rect, camera, &wire.points, style);
}

pub(crate) fn wire_style(
    color: egui::Color32,
    class: NetClass,
    flow_phase: Option<f32>,
) -> WireStyle {
    WireStyle {
        color,
        width: 2.0,
        dash: wire_dash(class),
        flow_phase,
    }
}

pub(crate) fn draw_wire_ends(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) {
    for point in [wire.points.first(), wire.points.last()]
        .into_iter()
        .flatten()
    {
        let screen = screen_pos(rect, camera, *point);
        painter.circle_stroke(screen, 5.0, egui::Stroke::new(1.5, SELECTED_STROKE));
    }
}

pub(crate) fn draw_junctions(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    document: &Document,
) {
    for point in document.junctions() {
        let screen = screen_pos(rect, camera, point);
        if !rect.contains(screen) {
            continue;
        }
        painter.circle_filled(screen, 3.2, JUNCTION_COLOR);
    }
}

pub(crate) fn draw_wire_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    if scene.wire_preview.is_empty() {
        return;
    }
    let color = net_color(scene.wire_class).gamma_multiply(0.8);
    let style = wire_style(color, scene.wire_class, None);
    draw_patterned_polyline(painter, rect, camera, scene.wire_preview, style);
    draw_preview_extension(painter, rect, camera, scene, hover, style);
}

pub(crate) fn draw_preview_extension(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
    mut style: WireStyle,
) {
    let Some((last, hover)) = preview_segment(scene, hover) else {
        return;
    };
    if !scene.document.can_add_segment(scene.library, last, hover) {
        style.color = PREVIEW_BAD;
    }
    draw_patterned_segment(painter, rect, camera, last, hover, style);
}

pub(crate) fn preview_segment(
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) -> Option<(GridPos, GridPos)> {
    let hover = hover?;
    let &last = scene.wire_preview.last()?;
    if last == hover {
        return None;
    }
    Some((last, hover))
}

pub(crate) fn draw_polyline(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    points: &[GridPos],
    color: egui::Color32,
    width: f32,
) {
    for segment in points.windows(2) {
        draw_segment(painter, rect, camera, segment[0], segment[1], color, width);
    }
}

pub(crate) fn draw_patterned_polyline(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    points: &[GridPos],
    style: WireStyle,
) {
    for segment in points.windows(2) {
        draw_patterned_segment(painter, rect, camera, segment[0], segment[1], style);
    }
}

pub(crate) fn draw_patterned_segment(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    style: WireStyle,
) {
    draw_wire_base(painter, rect, camera, start, end, style);
    let Some(phase) = style.flow_phase else {
        return;
    };
    let [start, end] = screen_segment(rect, camera, start, end);
    paint_flow_overlay(painter, start, end, phase);
}

pub(crate) fn draw_wire_base(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    style: WireStyle,
) {
    let Some((dash_length, gap_length)) = style.dash else {
        draw_segment(painter, rect, camera, start, end, style.color, style.width);
        return;
    };
    let [start, end] = screen_segment(rect, camera, start, end);
    painter.extend(egui::Shape::dashed_line(
        &[start, end],
        egui::Stroke::new(style.width, style.color),
        dash_length,
        gap_length,
    ));
}

pub(crate) fn paint_flow_overlay(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    phase: f32,
) {
    painter.extend(egui::Shape::dashed_line_with_offset(
        &[start, end],
        egui::Stroke::new(1.5, egui::Color32::from_white_alpha(90)),
        &[6.0],
        &[10.0],
        phase,
    ));
}

pub(crate) fn screen_segment(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> [egui::Pos2; 2] {
    [
        screen_pos(rect, camera, start),
        screen_pos(rect, camera, end),
    ]
}

pub(crate) fn wire_dash(class: NetClass) -> Option<(f32, f32)> {
    match class {
        NetClass::Power | NetClass::Ground | NetClass::Digital => None,
        NetClass::Analog => Some((6.0, 4.0)),
        NetClass::Clock => Some((10.0, 3.0)),
        NetClass::Uart => Some((2.0, 3.0)),
        NetClass::Spi => Some((8.0, 3.0)),
        NetClass::I2c => Some((4.0, 4.0)),
        NetClass::Pwm => Some((12.0, 3.0)),
        NetClass::Reset => Some((3.0, 5.0)),
        NetClass::User => Some((5.0, 5.0)),
    }
}

pub(crate) fn draw_segment(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
    color: egui::Color32,
    width: f32,
) {
    let start = screen_pos(rect, camera, start);
    let end = screen_pos(rect, camera, end);
    painter.line_segment([start, end], egui::Stroke::new(width, color));
}

pub(crate) fn wire_color(wire: &crate::model::Wire) -> egui::Color32 {
    wire.color_override.map_or_else(
        || net_color(wire.class),
        |[red, green, blue]| egui::Color32::from_rgb(red, green, blue),
    )
}

pub(crate) fn wire_label_text(wire: &crate::model::Wire) -> &str {
    wire.label
        .as_deref()
        .filter(|label| !label.is_empty())
        .unwrap_or(wire.class.label())
}

pub(crate) fn net_color(class: NetClass) -> egui::Color32 {
    match class {
        NetClass::Power => egui::Color32::from_rgb(225, 85, 85),
        NetClass::Ground => egui::Color32::from_rgb(125, 130, 140),
        NetClass::Digital => egui::Color32::from_rgb(85, 205, 125),
        NetClass::Analog => egui::Color32::from_rgb(90, 155, 240),
        NetClass::Clock => egui::Color32::from_rgb(240, 165, 75),
        NetClass::Uart => egui::Color32::from_rgb(80, 215, 225),
        NetClass::Spi => egui::Color32::from_rgb(185, 125, 235),
        NetClass::I2c => egui::Color32::from_rgb(230, 210, 95),
        NetClass::Pwm => egui::Color32::from_rgb(230, 115, 195),
        NetClass::Reset => egui::Color32::from_rgb(185, 135, 95),
        NetClass::User => egui::Color32::from_rgb(195, 200, 210),
    }
}

pub(crate) fn draw_library_preview(
    painter: &egui::Painter,
    area: egui::Rect,
    definition: &ComponentDefinition,
) {
    let body = preview_rect(area, definition);
    draw_body(painter, body, egui::Stroke::new(1.0, BODY_STROKE));
    draw_symbol(painter, body, definition.symbol);
}

pub(crate) fn preview_rect(area: egui::Rect, definition: &ComponentDefinition) -> egui::Rect {
    let width = definition.size.width as f32 + 2.0;
    let height = definition.size.height as f32 + 2.0;
    let scale = (area.width() / width)
        .min(area.height() / height)
        .clamp(1.0, 24.0);
    let size = egui::vec2(
        definition.size.width as f32 * scale,
        definition.size.height as f32 * scale,
    );
    egui::Rect::from_center_size(area.center(), size)
}

pub(crate) fn wire_visible(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    wire: &crate::model::Wire,
) -> bool {
    wire.points
        .windows(2)
        .any(|segment| segment_visible(rect, camera, segment[0], segment[1]))
}

pub(crate) fn segment_visible(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    start: GridPos,
    end: GridPos,
) -> bool {
    let start = screen_pos(rect, camera, start);
    let end = screen_pos(rect, camera, end);
    egui::Rect::from_two_pos(start, end).intersects(rect)
}
