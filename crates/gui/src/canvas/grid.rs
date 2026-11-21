use super::*;
use crate::design;

pub(super) fn draw_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    logical_step: u32,
    palette: &design::Palette,
) {
    let step = grid_step(camera.scale(), logical_step);
    let range = camera.visible_range(rect, step);
    draw_minor_dots(painter, rect, camera, range, step, palette);
    draw_major_lines(painter, rect, camera, range, step, palette);
    draw_axes(painter, rect, camera, palette);
    draw_rulers(painter, rect, camera, range, step, palette);
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

fn draw_minor_dots(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    for x in range.min_x..=range.max_x {
        for y in range.min_y..=range.max_y {
            let world = egui::pos2((x * step) as f32, (y * step) as f32);
            let screen = camera.world_to_screen(rect, world);
            painter.circle_filled(screen, 1.0, palette.grid_minor);
        }
    }
}

fn draw_major_lines(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    range: GridRange,
    step: i32,
    palette: &design::Palette,
) {
    let major = step * 10;
    let stroke = egui::Stroke::new(1.0, palette.grid_major);
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
