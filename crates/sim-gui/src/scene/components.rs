use super::*;

pub(crate) fn draw_components(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    for instance in scene.document.components() {
        draw_component_instance(painter, rect, camera, scene, instance, hover);
    }
}

pub(crate) fn draw_component_instance(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    instance: &crate::model::ComponentInstance,
    hover: Option<GridPos>,
) {
    let Some(definition) = library::find(instance.definition) else {
        return;
    };
    let body = component_rect(rect, camera, instance, definition);
    if !rect.intersects(body) {
        return;
    }
    let selected = scene.selection.contains_component(instance.id);
    draw_component(painter, rect, camera, definition, instance, selected, hover);
}

pub(crate) fn draw_component(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    selected: bool,
    hover: Option<GridPos>,
) {
    let body = component_rect(rect, camera, instance, definition);
    draw_body(painter, body, component_stroke(selected));
    draw_symbol(painter, body, definition.symbol);
    draw_pins(painter, rect, camera, definition, instance, hover);
    if show_component_labels(selected, camera.zoom) {
        draw_labels(painter, body, instance);
    }
}

pub(crate) fn show_component_labels(selected: bool, zoom: f32) -> bool {
    selected || zoom >= 0.7
}

pub(crate) fn component_stroke(selected: bool) -> egui::Stroke {
    if selected {
        egui::Stroke::new(2.0, SELECTED_STROKE)
    } else {
        egui::Stroke::new(1.0, BODY_STROKE)
    }
}

pub(crate) fn draw_body(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    painter.rect_filled(body, egui::CornerRadius::ZERO, BODY_FILL);
    painter.rect_stroke(
        body,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn draw_symbol(
    painter: &egui::Painter,
    body: egui::Rect,
    symbol: crate::model::SymbolKind,
) {
    let stroke = egui::Stroke::new(1.5, SYMBOL_STROKE);
    match symbol {
        crate::model::SymbolKind::Resistor => draw_resistor(painter, body, stroke),
        crate::model::SymbolKind::Led => draw_led(painter, body, stroke),
        crate::model::SymbolKind::Button => draw_button(painter, body, stroke),
        crate::model::SymbolKind::Vcc => draw_vcc(painter, body, stroke),
        crate::model::SymbolKind::Gnd => draw_gnd(painter, body, stroke),
        crate::model::SymbolKind::Mcu => draw_mcu(painter, body, stroke),
    }
}

pub(crate) fn draw_resistor(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let inset = body.shrink2(egui::vec2(body.width() * 0.18, body.height() * 0.22));
    painter.rect_stroke(
        inset,
        egui::CornerRadius::ZERO,
        stroke,
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn draw_led(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    let top = center + egui::vec2(0.0, -body.height() * 0.22);
    let bottom = center + egui::vec2(0.0, body.height() * 0.22);
    let left = center + egui::vec2(-body.width() * 0.16, 0.0);
    let right = center + egui::vec2(body.width() * 0.16, 0.0);
    painter.add(egui::Shape::convex_polygon(
        vec![left, right, bottom],
        SYMBOL_STROKE,
        stroke,
    ));
    painter.line_segment([top, bottom], stroke);
}

pub(crate) fn draw_button(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    let left = center + egui::vec2(-body.width() * 0.18, 0.0);
    let right = center + egui::vec2(body.width() * 0.18, 0.0);
    let top = center + egui::vec2(0.0, -body.height() * 0.2);
    painter.line_segment([left, right], stroke);
    painter.line_segment(
        [top + egui::vec2(-6.0, 0.0), top + egui::vec2(6.0, 0.0)],
        stroke,
    );
}

pub(crate) fn draw_vcc(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    painter.circle_stroke(center, body.height() * 0.18, stroke);
    painter.line_segment(
        [
            center + egui::vec2(0.0, body.height() * 0.18),
            center + egui::vec2(0.0, body.height() * 0.4),
        ],
        stroke,
    );
}

pub(crate) fn draw_gnd(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let center = body.center();
    for (index, width) in [0.35, 0.24, 0.13].iter().enumerate() {
        let y = center.y + index as f32 * 6.0 - 6.0;
        painter.line_segment(
            [
                egui::pos2(center.x - body.width() * width, y),
                egui::pos2(center.x + body.width() * width, y),
            ],
            stroke,
        );
    }
}

pub(crate) fn draw_mcu(painter: &egui::Painter, body: egui::Rect, stroke: egui::Stroke) {
    let notch = body.center_top() + egui::vec2(-body.width() * 0.12, 0.0);
    painter.circle_stroke(notch, body.width() * 0.08, stroke);
    painter.line_segment(
        [
            body.center_top() + egui::vec2(0.0, body.height() * 0.08),
            body.center_bottom() - egui::vec2(0.0, body.height() * 0.08),
        ],
        egui::Stroke::new(1.0, BODY_STROKE),
    );
}

pub(crate) fn draw_pins(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    hover: Option<GridPos>,
) {
    for pin in definition.pins {
        draw_pin(painter, rect, camera, definition, instance, pin, hover);
    }
}

pub(crate) fn draw_pin(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    definition: &ComponentDefinition,
    instance: &crate::model::ComponentInstance,
    pin: &crate::model::PinDefinition,
    hover: Option<GridPos>,
) {
    let world = world_pin(instance, definition, pin);
    let screen = screen_pos(rect, camera, world);
    let direction = pin_direction(definition, pin, instance.rotation, instance.mirror);
    let stub = pin_stub(rect, camera, world, direction);
    painter.line_segment([screen, stub], egui::Stroke::new(1.5, PIN_COLOR));
    painter.circle_filled(stub, 2.0, PIN_COLOR);
    let hovered = hover == Some(world) || hover == Some(world.offset(direction));
    draw_pin_details(painter, stub, pin, direction, camera.zoom, hovered);
}

pub(crate) fn draw_pin_details(
    painter: &egui::Painter,
    stub: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
    zoom: f32,
    hovered: bool,
) {
    if zoom >= 0.75 {
        draw_pin_label(painter, stub, pin, direction);
    }
    if hovered {
        draw_pin_tooltip(painter, stub, pin, direction);
    }
}

pub(crate) fn draw_pin_tooltip(
    painter: &egui::Painter,
    position: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
) {
    let text = pin_tooltip(pin, direction);
    let pos = position + egui::vec2(8.0, -22.0);
    let galley = painter.layout_no_wrap(text, egui::FontId::monospace(10.0), TEXT_COLOR);
    let background = egui::Rect::from_min_size(pos, galley.size()).expand(3.0);
    painter.rect_filled(background, 3.0, egui::Color32::from_black_alpha(210));
    painter.galley(pos, galley, TEXT_COLOR);
}

pub(crate) fn pin_tooltip(pin: &crate::model::PinDefinition, direction: GridPos) -> String {
    let side = match (direction.x, direction.y) {
        (-1, 0) => "left",
        (1, 0) => "right",
        (0, -1) => "up",
        (0, 1) => "down",
        _ => "pin",
    };
    format!("{} ({}) {side}", pin.name, pin.number)
}

pub(crate) fn pin_stub(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    world: GridPos,
    direction: GridPos,
) -> egui::Pos2 {
    let stub = GridPos::new(world.x + direction.x, world.y + direction.y);
    screen_pos(rect, camera, stub)
}

pub(crate) fn draw_pin_label(
    painter: &egui::Painter,
    position: egui::Pos2,
    pin: &crate::model::PinDefinition,
    direction: GridPos,
) {
    let offset = egui::vec2(direction.x as f32 * 5.0, direction.y as f32 * 5.0);
    painter.text(
        position + offset,
        pin_anchor(direction),
        pin_label_text(pin),
        egui::FontId::monospace(9.0),
        TEXT_COLOR,
    );
}

pub(crate) fn pin_label_text(pin: &crate::model::PinDefinition) -> String {
    if pin.name == pin.number {
        return pin.name.to_string();
    }
    format!("{}/{}", pin.name, pin.number)
}

pub(crate) fn pin_anchor(direction: GridPos) -> egui::Align2 {
    match (direction.x, direction.y) {
        (-1, _) => egui::Align2::RIGHT_CENTER,
        (1, _) => egui::Align2::LEFT_CENTER,
        (_, -1) => egui::Align2::CENTER_BOTTOM,
        _ => egui::Align2::CENTER_TOP,
    }
}

pub(crate) fn draw_labels(
    painter: &egui::Painter,
    body: egui::Rect,
    instance: &crate::model::ComponentInstance,
) {
    let text = format!("{} {}", instance.label, instance.value);
    painter.text(
        body.center_bottom() + egui::vec2(0.0, 4.0),
        egui::Align2::CENTER_TOP,
        text,
        egui::FontId::monospace(10.0),
        TEXT_COLOR,
    );
}

pub(crate) fn draw_ghost(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    definition: &ComponentDefinition,
    hover: GridPos,
) {
    let position = placement_position(definition, hover, scene.rotation);
    let color = ghost_color(scene, definition, position);
    let body = rect_for(
        rect,
        camera,
        position,
        body_size(definition, scene.rotation),
    );
    draw_ghost_body(painter, body, color);
}

pub(crate) fn ghost_color(
    scene: &Scene<'_>,
    definition: &ComponentDefinition,
    position: GridPos,
) -> egui::Color32 {
    let valid = scene.document.can_place(
        scene.library,
        definition,
        position,
        scene.rotation,
        scene.mirror,
    );
    if valid { GHOST_OK } else { GHOST_BAD }
}

pub(crate) fn draw_ghost_body(painter: &egui::Painter, body: egui::Rect, color: egui::Color32) {
    painter.rect_filled(body, egui::CornerRadius::ZERO, color.gamma_multiply(0.18));
    painter.rect_stroke(
        body,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(2.0, color),
        egui::StrokeKind::Middle,
    );
}

pub(crate) fn component_rect(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    instance: &crate::model::ComponentInstance,
    definition: &ComponentDefinition,
) -> egui::Rect {
    rect_for(
        rect,
        camera,
        instance.position,
        body_size(definition, instance.rotation),
    )
}

pub(crate) fn rect_for(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    position: GridPos,
    size: crate::model::GridSize,
) -> egui::Rect {
    let min = camera.world_to_screen(rect, egui::pos2(position.x as f32, position.y as f32));
    let max = camera.world_to_screen(
        rect,
        egui::pos2(
            (position.x + size.width) as f32,
            (position.y + size.height) as f32,
        ),
    );
    egui::Rect::from_min_max(min, max)
}

pub(crate) fn screen_pos(
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    position: GridPos,
) -> egui::Pos2 {
    camera.world_to_screen(rect, egui::pos2(position.x as f32, position.y as f32))
}
