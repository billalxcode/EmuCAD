use super::*;
use crate::design;

pub(super) fn draw_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    options: GridOptions,
    palette: &design::Palette,
) {
    let step = grid_step(camera.scale(), options.step);
    let range = camera.visible_range(rect, step);
    if options.visible {
        draw_grid_markers(painter, rect, camera, range, step, options.style, palette);
    }
    if options.axes {
        draw_axes(painter, rect, camera, palette);
    }
    if options.rulers {
        draw_rulers(painter, rect, camera, range, step, palette);
    }
}

fn draw_grid_markers(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    style: GridStyle,
    palette: &design::Palette,
) {
    match style {
        GridStyle::Dots => draw_dots(painter, rect, camera, range, step, palette),
        GridStyle::Lines => draw_lines(painter, rect, camera, range, step, palette),
        GridStyle::Cross => draw_crosses(painter, rect, camera, range, step, palette),
    }
}

fn draw_dots(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    for x in range.min_x..=range.max_x {
        for y in range.min_y..=range.max_y {
            draw_dot(painter, rect, camera, x, y, step, palette);
        }
    }
}

fn draw_dot(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    x: i32,
    y: i32,
    step: i32,
    palette: &design::Palette,
) {
    let world = egui::pos2((x * step) as f32, (y * step) as f32);
    let screen = camera.world_to_screen(rect, world);
    let major = is_major(x) && is_major(y);
    let radius = if major { 2.0 } else { 1.0 };
    let color = if major {
        palette.grid_major
    } else {
        palette.grid_minor
    };
    painter.circle_filled(screen, radius, color);
}

fn draw_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    for index in range.min_x..=range.max_x {
        let stroke = line_stroke(is_major(index), palette);
        draw_vertical_line(painter, rect, camera, (index * step) as f32, stroke);
    }
    for index in range.min_y..=range.max_y {
        let stroke = line_stroke(is_major(index), palette);
        draw_horizontal_line(painter, rect, camera, (index * step) as f32, stroke);
    }
}

fn line_stroke(major: bool, palette: &design::Palette) -> egui::Stroke {
    if major {
        egui::Stroke::new(2.0, palette.grid_major)
    } else {
        egui::Stroke::new(1.0, palette.grid_minor)
    }
}

fn draw_crosses(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    for x in range.min_x..=range.max_x {
        for y in range.min_y..=range.max_y {
            draw_cross(painter, rect, camera, x, y, step, palette);
        }
    }
}

fn draw_cross(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    x: i32,
    y: i32,
    step: i32,
    palette: &design::Palette,
) {
    let screen = cross_screen(rect, camera, x, y, step);
    let major = is_major(x) && is_major(y);
    let (size, color) = cross_style(major, palette);
    paint_cross(painter, screen, size, egui::Stroke::new(1.0, color));
}

fn cross_screen(rect: egui::Rect, camera: &GridCamera, x: i32, y: i32, step: i32) -> egui::Pos2 {
    let world = egui::pos2((x * step) as f32, (y * step) as f32);
    camera.world_to_screen(rect, world)
}

fn cross_style(major: bool, palette: &design::Palette) -> (f32, egui::Color32) {
    if major {
        (4.0, palette.grid_major)
    } else {
        (2.0, palette.grid_minor)
    }
}

fn paint_cross(painter: &egui::Painter, screen: egui::Pos2, size: f32, stroke: egui::Stroke) {
    painter.line_segment(
        [
            screen - egui::vec2(size, 0.0),
            screen + egui::vec2(size, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            screen - egui::vec2(0.0, size),
            screen + egui::vec2(0.0, size),
        ],
        stroke,
    );
}

fn is_major(index: i32) -> bool {
    index.rem_euclid(10) == 0
}

fn draw_rulers(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    draw_x_ruler(painter, rect, camera, range, step, palette);
    draw_y_ruler(painter, rect, camera, range, step, palette);
}

fn draw_x_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    let major = step * 10;
    let font = design::mono(10.0);
    for index in (range.min_x..=range.max_x).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(world, 0.0));
        let pos = egui::pos2(screen.x + 2.0, rect.top() + 2.0);
        paint_ruler_text(painter, pos, world, &font, palette);
    }
}

fn draw_y_ruler(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    let major = step * 10;
    let font = design::mono(10.0);
    for index in (range.min_y..=range.max_y).filter(|index| index * step % major == 0) {
        let world = (index * step) as f32;
        let screen = camera.world_to_screen(rect, egui::pos2(0.0, world));
        let pos = egui::pos2(rect.left() + 2.0, screen.y + 2.0);
        paint_ruler_text(painter, pos, world, &font, palette);
    }
}

fn paint_ruler_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    world: f32,
    font: &egui::FontId,
    palette: &design::Palette,
) {
    painter.text(
        pos,
        egui::Align2::LEFT_TOP,
        format!("{world:.0}"),
        font.clone(),
        palette.ruler,
    );
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

fn draw_axes(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    palette: &design::Palette,
) {
    let origin = camera.world_to_screen(rect, egui::Pos2::ZERO);
    let stroke = egui::Stroke::new(1.0, palette.grid_axis);
    draw_vertical_axis(painter, rect, origin.x, stroke);
    draw_horizontal_axis(painter, rect, origin.y, stroke);
}

fn draw_vertical_axis(painter: &egui::Painter, rect: egui::Rect, x: f32, stroke: egui::Stroke) {
    painter.line_segment(
        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
        stroke,
    );
}

fn draw_horizontal_axis(painter: &egui::Painter, rect: egui::Rect, y: f32, stroke: egui::Stroke) {
    painter.line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        stroke,
    );
}

pub(super) fn grid_step(scale: f32, logical_step: u32) -> i32 {
    let mut step = logical_step.max(1) as i32;
    if scale <= 0.0 {
        return step;
    }
    while (step as f32 * scale) < 10.0 {
        step *= 10;
    }
    step
}
