use super::*;
use crate::design;

struct GridPaint<'a> {
    painter: &'a egui::Painter,
    rect: egui::Rect,
    camera: &'a GridCamera,
    range: GridRange,
    step: i32,
    width: f32,
    palette: &'a design::Palette,
}

impl<'a> GridPaint<'a> {
    fn new(
        painter: &'a egui::Painter,
        rect: egui::Rect,
        camera: &'a GridCamera,
        range: GridRange,
        step: i32,
        options: GridOptions,
        palette: &'a design::Palette,
    ) -> Self {
        Self {
            painter,
            rect,
            camera,
            range,
            step,
            width: options.line_width,
            palette,
        }
    }
}

pub(crate) fn draw_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    options: GridOptions,
    palette: &design::Palette,
) {
    let step = grid_step(camera.scale(), options.step, options.min_spacing);
    let range = camera.visible_range(rect, step);
    let paint = GridPaint::new(painter, rect, camera, range, step, options, palette);
    if options.visible {
        draw_grid_markers(&paint, options.style);
    }
    if options.axes {
        draw_axes(&paint);
    }
    if options.rulers {
        draw_rulers(&paint);
    }
}

fn draw_grid_markers(paint: &GridPaint<'_>, style: GridStyle) {
    match style {
        GridStyle::Dots => draw_dots(paint),
        GridStyle::Lines => draw_lines(paint),
        GridStyle::Cross => draw_crosses(paint),
    }
}

fn draw_dots(paint: &GridPaint<'_>) {
    for x in paint.range.min_x..=paint.range.max_x {
        for y in paint.range.min_y..=paint.range.max_y {
            draw_dot(paint, x, y);
        }
    }
}

fn draw_dot(paint: &GridPaint<'_>, x: i32, y: i32) {
    let screen = grid_screen(paint, x, y);
    let major = is_major(x) && is_major(y);
    let radius = (paint.width * if major { 2.0 } else { 1.0 }).max(1.0);
    let color = if major {
        paint.palette.grid_major
    } else {
        paint.palette.grid_minor
    };
    paint.painter.circle_filled(screen, radius, color);
}

fn draw_lines(paint: &GridPaint<'_>) {
    for index in paint.range.min_x..=paint.range.max_x {
        let stroke = line_stroke(paint, is_major(index));
        draw_vertical_line(paint, (index * paint.step) as f32, stroke);
    }
    for index in paint.range.min_y..=paint.range.max_y {
        let stroke = line_stroke(paint, is_major(index));
        draw_horizontal_line(paint, (index * paint.step) as f32, stroke);
    }
}

fn line_stroke(paint: &GridPaint<'_>, major: bool) -> egui::Stroke {
    if major {
        egui::Stroke::new(paint.width * 2.0, paint.palette.grid_major)
    } else {
        egui::Stroke::new(paint.width, paint.palette.grid_minor)
    }
}

fn draw_crosses(paint: &GridPaint<'_>) {
    for x in paint.range.min_x..=paint.range.max_x {
        for y in paint.range.min_y..=paint.range.max_y {
            draw_cross(paint, x, y);
        }
    }
}

fn draw_cross(paint: &GridPaint<'_>, x: i32, y: i32) {
    let screen = grid_screen(paint, x, y);
    let major = is_major(x) && is_major(y);
    let (size, color) = cross_style(paint, major);
    paint_cross(paint.painter, screen, size, egui::Stroke::new(1.0, color));
}

fn grid_screen(paint: &GridPaint<'_>, x: i32, y: i32) -> egui::Pos2 {
    let world = egui::pos2((x * paint.step) as f32, (y * paint.step) as f32);
    paint.camera.world_to_screen(paint.rect, world)
}

fn cross_style(paint: &GridPaint<'_>, major: bool) -> (f32, egui::Color32) {
    let base = (paint.width * 2.0).max(2.0);
    if major {
        (base * 2.0, paint.palette.grid_major)
    } else {
        (base, paint.palette.grid_minor)
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

pub(super) fn is_major(index: i32) -> bool {
    index.rem_euclid(10) == 0
}

fn draw_rulers(paint: &GridPaint<'_>) {
    draw_x_ruler(paint);
    draw_y_ruler(paint);
}

fn draw_x_ruler(paint: &GridPaint<'_>) {
    let major = paint.step * 10;
    let font = design::mono(10.0);
    let range = paint.range.min_x..=paint.range.max_x;
    for index in range.filter(|index| index * paint.step % major == 0) {
        let world = (index * paint.step) as f32;
        let screen = paint
            .camera
            .world_to_screen(paint.rect, egui::pos2(world, 0.0));
        let pos = egui::pos2(screen.x + 2.0, paint.rect.top() + 2.0);
        paint_ruler_text(paint.painter, pos, world, &font, paint.palette);
    }
}

fn draw_y_ruler(paint: &GridPaint<'_>) {
    let major = paint.step * 10;
    let font = design::mono(10.0);
    let range = paint.range.min_y..=paint.range.max_y;
    for index in range.filter(|index| index * paint.step % major == 0) {
        let world = (index * paint.step) as f32;
        let screen = paint
            .camera
            .world_to_screen(paint.rect, egui::pos2(0.0, world));
        let pos = egui::pos2(paint.rect.left() + 2.0, screen.y + 2.0);
        paint_ruler_text(paint.painter, pos, world, &font, paint.palette);
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

fn draw_vertical_line(paint: &GridPaint<'_>, world_x: f32, stroke: egui::Stroke) {
    let screen = paint
        .camera
        .world_to_screen(paint.rect, egui::pos2(world_x, 0.0));
    paint.painter.line_segment(
        [
            egui::pos2(screen.x, paint.rect.top()),
            egui::pos2(screen.x, paint.rect.bottom()),
        ],
        stroke,
    );
}

fn draw_horizontal_line(paint: &GridPaint<'_>, world_y: f32, stroke: egui::Stroke) {
    let screen = paint
        .camera
        .world_to_screen(paint.rect, egui::pos2(0.0, world_y));
    paint.painter.line_segment(
        [
            egui::pos2(paint.rect.left(), screen.y),
            egui::pos2(paint.rect.right(), screen.y),
        ],
        stroke,
    );
}

fn draw_axes(paint: &GridPaint<'_>) {
    let origin = paint.camera.world_to_screen(paint.rect, egui::Pos2::ZERO);
    let stroke = egui::Stroke::new(1.0, paint.palette.grid_axis);
    draw_vertical_axis(paint, origin.x, stroke);
    draw_horizontal_axis(paint, origin.y, stroke);
}

fn draw_vertical_axis(paint: &GridPaint<'_>, x: f32, stroke: egui::Stroke) {
    paint.painter.line_segment(
        [
            egui::pos2(x, paint.rect.top()),
            egui::pos2(x, paint.rect.bottom()),
        ],
        stroke,
    );
}

fn draw_horizontal_axis(paint: &GridPaint<'_>, y: f32, stroke: egui::Stroke) {
    paint.painter.line_segment(
        [
            egui::pos2(paint.rect.left(), y),
            egui::pos2(paint.rect.right(), y),
        ],
        stroke,
    );
}

pub(super) fn grid_step(scale: f32, logical_step: u32, min_spacing: f32) -> i32 {
    let mut step = logical_step.max(1) as i32;
    if scale <= 0.0 {
        return step;
    }
    let min_spacing = min_spacing.max(1.0);
    while (step as f32 * scale) < min_spacing {
        step *= 10;
    }
    step
}
