use super::*;

pub(super) fn draw_grid(painter: &egui::Painter, rect: egui::Rect, camera: &GridCamera) {
    let step = grid_step(camera.scale());
    let range = camera.visible_range(rect, step);
    draw_minor_dots(painter, rect, camera, range, step);
    draw_major_lines(painter, rect, camera, range, step);
    draw_axes(painter, rect, camera);
    draw_rulers(painter, rect, camera, range, step);
}

fn draw_rulers(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    draw_x_ruler(painter, rect, camera, range, step);
    draw_y_ruler(painter, rect, camera, range, step);
}

fn draw_x_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let font = egui::FontId::monospace(9.0);
    for index in (range.min_x..=range.max_x).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(world, 0.0));
        let pos = egui::pos2(screen.x + 2.0, rect.top() + 2.0);
        paint_ruler_text(painter, pos, world, &font);
    }
}

fn draw_y_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let font = egui::FontId::monospace(9.0);
    for index in (range.min_y..=range.max_y).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(0.0, world));
        let pos = egui::pos2(rect.left() + 2.0, screen.y + 2.0);
        paint_ruler_text(painter, pos, world, &font);
    }
}

fn paint_ruler_text(painter: &egui::Painter, pos: egui::Pos2, world: f32, font: &egui::FontId) {
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        format!("{world:.0}"),
        font.clone(),
        RULER_COLOR,
    );
}

fn draw_minor_dots(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    for x in range.min_x..=range.max_x {
        for y in range.min_y..=range.max_y {
            let world = egui::pos2((x * step) as f32, (y * step) as f32);
            let screen = camera.world_to_screen(rect, world);
            painter.circle_filled(screen, 1.0, MINOR_DOT);
        }
    }
}

fn draw_major_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
) {
    let major = step * 5;
    let stroke = egui::Stroke::new(1.0, MAJOR_LINE);
    for index in (range.min_x..=range.max_x).filter(|index| index * step % major == 0) {
        draw_vertical_line(painter, rect, camera, (index * step) as f32, stroke);
    }
    for index in (range.min_y..=range.max_y).filter(|index| index * step % major == 0) {
        draw_horizontal_line(painter, rect, camera, (index * step) as f32, stroke);
    }
}

fn draw_vertical_line(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    world_x: f32,
    stroke: egui::Stroke,
) {
    let screen = camera.world_to_screen(rect, egui::pos2(world_x, 0.0));
    painter.line_segment(
        [
            egui::pos2(screen.x, rect.top()),
            egui::pos2(screen.x, rect.bottom()),
        ],
        stroke,
    );
}

fn draw_horizontal_line(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    world_y: f32,
    stroke: egui::Stroke,
) {
    let screen = camera.world_to_screen(rect, egui::pos2(0.0, world_y));
    painter.line_segment(
        [
            egui::pos2(rect.left(), screen.y),
            egui::pos2(rect.right(), screen.y),
        ],
        stroke,
    );
}

fn draw_axes(painter: &egui::Painter, rect: egui::Rect, camera: &GridCamera) {
    let origin = camera.world_to_screen(rect, egui::Pos2::ZERO);
    let stroke = egui::Stroke::new(1.0, AXIS_LINE);
    painter.line_segment(
        [
            egui::pos2(origin.x, rect.top()),
            egui::pos2(origin.x, rect.bottom()),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(rect.left(), origin.y),
            egui::pos2(rect.right(), origin.y),
        ],
        stroke,
    );
}

pub(super) fn grid_step(scale: f32) -> i32 {
    if scale >= 12.0 {
        1
    } else if scale >= 6.0 {
        2
    } else {
        5
    }
}
