//! Headless RGBA golden renderer for GUI regression tests.

use crate::canvas::DVec2;
use crate::design;
use crate::model::{Document, GridPos, Mirror, Rotation};
use crate::scene::{self, Scene};
use crate::selection::Selection;
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Color32>,
}

/// Renders the fixed golden scene into straight RGBA bytes.
pub fn render_golden() -> Vec<u8> {
    render_with(WIDTH, HEIGHT, draw_golden)
}

/// Renders the fixed grid golden into straight RGBA bytes.
pub fn render_grid_golden() -> Vec<u8> {
    render_with(WIDTH, HEIGHT, draw_grid_golden)
}

/// Renders the toolbar golden into straight RGBA bytes.
pub fn render_toolbar_golden() -> Vec<u8> {
    render_with(2400, 120, draw_toolbar_golden)
}

fn render_with(width: usize, height: usize, draw: impl Fn(&mut egui::Ui)) -> Vec<u8> {
    let context = egui::Context::default();
    design::install_fonts(&context);
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            Vec2::new(width as f32, height as f32),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input, draw);
    let textures = collect_textures(&output.textures_delta);
    output.textures_delta.clear();
    let primitives = context.tessellate(output.shapes, 1.0);
    rasterize(primitives, &textures, width, height)
}

fn draw_golden(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, design::DARK.canvas);
    draw_golden_scene(painter, rect);
    golden_caption(painter, "EmuCAD golden");
}

fn draw_grid_golden(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, design::DARK.canvas);
    let options = crate::canvas::GridOptions {
        style: crate::canvas::GridStyle::Dots,
        axes: true,
        rulers: true,
        ..Default::default()
    };
    crate::canvas::draw_grid(painter, rect, &golden_camera(), options, &design::DARK);
    golden_caption(painter, "EmuCAD grid golden");
}

fn draw_toolbar_golden(ui: &mut egui::Ui) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, design::DARK.canvas);
    let mut app = crate::app::GuiApp::default();
    app.toolbar(ui);
}

fn golden_caption(painter: &egui::Painter, text: &str) {
    painter.text(
        egui::pos2(10.0, 216.0),
        egui::Align2::LEFT_TOP,
        text,
        design::mono(12.0),
        design::DARK.text,
    );
}

fn draw_golden_scene(painter: &egui::Painter, rect: Rect) {
    let document = golden_document();
    let selection = Selection::default();
    let camera = golden_camera();
    let scene = golden_scene(&document, &selection);
    scene::draw(painter, rect, &camera, scene, None);
}

fn golden_camera() -> crate::canvas::GridCamera {
    crate::canvas::GridCamera {
        pan: DVec2::new(14.0, 10.0),
        zoom: 0.7,
        cell_px: 17.0,
        zoom_history: Vec::new(),
    }
}

fn golden_scene<'a>(document: &'a Document, selection: &'a Selection) -> Scene<'a> {
    Scene::new((document, crate::library::builtin(), selection))
}

fn golden_document() -> Document {
    let mut document = Document::new();
    add_symbol(&mut document, "resistor", 0, 2);
    add_symbol(&mut document, "led", 6, 2);
    add_symbol(&mut document, "button", 12, 2);
    add_symbol(&mut document, "switch.mechanical.spst_switch", 18, 2);
    add_symbol(&mut document, "transistor.bjt_npn.npn_small_signal", 2, 9);
    add_symbol(&mut document, "analog.opamp.ideal_opamp", 10, 9);
    add_symbol(&mut document, "switch.mechanical.spdt_switch", 18, 9);
    document
}

fn add_symbol(document: &mut Document, id: &str, x: i32, y: i32) {
    let definition = crate::library::find(id).expect(id);
    document.add(definition, GridPos::new(x, y), Rotation::Deg0, Mirror::None);
}

fn collect_textures(delta: &egui::TexturesDelta) -> HashMap<egui::TextureId, Texture> {
    let mut textures = HashMap::new();
    for (id, images) in &delta.set {
        for image in images {
            apply_delta(&mut textures, *id, image);
        }
    }
    textures
}

fn apply_delta(
    textures: &mut HashMap<egui::TextureId, Texture>,
    id: egui::TextureId,
    delta: &egui::epaint::ImageDelta,
) {
    let egui::epaint::ImageData::Color(image) = &delta.image;
    let Some([x, y]) = delta.pos else {
        textures.insert(id, texture_from(image.as_ref()));
        return;
    };
    if let Some(texture) = textures.get_mut(&id) {
        blit(texture, x, y, image.as_ref());
    }
}

fn texture_from(image: &egui::ColorImage) -> Texture {
    Texture {
        width: image.size[0],
        height: image.size[1],
        pixels: image.pixels.clone(),
    }
}

fn blit(texture: &mut Texture, x: usize, y: usize, image: &egui::ColorImage) {
    for row in 0..image.size[1] {
        for column in 0..image.size[0] {
            let index = (y + row) * texture.width + x + column;
            texture.pixels[index] = image.pixels[row * image.size[0] + column];
        }
    }
}

fn rasterize(
    primitives: Vec<egui::ClippedPrimitive>,
    textures: &HashMap<egui::TextureId, Texture>,
    width: usize,
    height: usize,
) -> Vec<u8> {
    let mut pixels = vec![[0.0_f32; 4]; width * height];
    for clipped in primitives {
        raster_clipped(&mut pixels, width, height, clipped, textures);
    }
    pixels.into_iter().flat_map(pixel_rgba).collect()
}

fn raster_clipped(
    pixels: &mut [[f32; 4]],
    width: usize,
    height: usize,
    clipped: egui::ClippedPrimitive,
    textures: &HashMap<egui::TextureId, Texture>,
) {
    let egui::epaint::Primitive::Mesh(mesh) = clipped.primitive else {
        return;
    };
    let texture = textures.get(&mesh.texture_id);
    for triangle in mesh.indices.chunks_exact(3) {
        let Some(vertices) = mesh_vertices(&mesh, triangle) else {
            continue;
        };
        raster_triangle(pixels, width, height, clipped.clip_rect, vertices, texture);
    }
}

fn mesh_vertices(mesh: &egui::epaint::Mesh, triangle: &[u32]) -> Option<[egui::epaint::Vertex; 3]> {
    let a = mesh.vertices.get(triangle[0] as usize)?;
    let b = mesh.vertices.get(triangle[1] as usize)?;
    let c = mesh.vertices.get(triangle[2] as usize)?;
    Some([*a, *b, *c])
}

fn raster_triangle(
    pixels: &mut [[f32; 4]],
    width: usize,
    height: usize,
    clip: Rect,
    vertices: [egui::epaint::Vertex; 3],
    texture: Option<&Texture>,
) {
    let [a, b, c] = vertices;
    let area = edge(a.pos, b.pos, c.pos);
    if area.abs() < f32::EPSILON {
        return;
    }
    let bounds = pixel_bounds(clip, a.pos, b.pos, c.pos, width, height);
    raster_area(pixels, width, area, [a, b, c], bounds, texture);
}

fn raster_area(
    pixels: &mut [[f32; 4]],
    width: usize,
    area: f32,
    vertices: [egui::epaint::Vertex; 3],
    bounds: (usize, usize, usize, usize),
    texture: Option<&Texture>,
) {
    let (min_x, min_y, max_x, max_y) = bounds;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let point = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
            raster_pixel(pixels, width, area, vertices, point, texture);
        }
    }
}

fn raster_pixel(
    pixels: &mut [[f32; 4]],
    width: usize,
    area: f32,
    vertices: [egui::epaint::Vertex; 3],
    point: Pos2,
    texture: Option<&Texture>,
) {
    let Some(weights) = barycentric(vertices, point, area) else {
        return;
    };
    let mut color = interpolate(vertices, weights);
    apply_texture(&mut color, vertices, weights, texture);
    blend(
        &mut pixels[point.y as usize * width + point.x as usize],
        color,
    );
}

fn barycentric(vertices: [egui::epaint::Vertex; 3], point: Pos2, area: f32) -> Option<[f32; 3]> {
    let [a, b, c] = vertices;
    let weights = [
        edge(b.pos, c.pos, point) / area,
        edge(c.pos, a.pos, point) / area,
        edge(a.pos, b.pos, point) / area,
    ];
    if weights.iter().any(|weight| *weight < -1e-4) {
        return None;
    }
    Some(weights)
}

fn apply_texture(
    color: &mut [f32; 4],
    vertices: [egui::epaint::Vertex; 3],
    weights: [f32; 3],
    texture: Option<&Texture>,
) {
    let Some(texture) = texture else {
        return;
    };
    let uv = interpolate_uv(vertices, weights);
    let texel = sample_texture(texture, uv);
    for channel in 0..4 {
        color[channel] *= texel[channel];
    }
}

fn pixel_bounds(
    clip: Rect,
    a: Pos2,
    b: Pos2,
    c: Pos2,
    width: usize,
    height: usize,
) -> (usize, usize, usize, usize) {
    let min = a.min(b).min(c).max(clip.min).max(Pos2::ZERO);
    let size = egui::pos2(width as f32, height as f32);
    let max = a.max(b).max(c).min(clip.max).min(size);
    let min_x = min.x.floor().max(0.0) as usize;
    let min_y = min.y.floor().max(0.0) as usize;
    let max_x = (max.x.ceil().max(0.0) as usize).min(width);
    let max_y = (max.y.ceil().max(0.0) as usize).min(height);
    (min_x, min_y, max_x, max_y)
}

fn edge(a: Pos2, b: Pos2, point: Pos2) -> f32 {
    (point.x - a.x) * (b.y - a.y) - (point.y - a.y) * (b.x - a.x)
}

fn interpolate(vertices: [egui::epaint::Vertex; 3], weights: [f32; 3]) -> [f32; 4] {
    let mut color = [0.0; 4];
    for (vertex, weight) in vertices.iter().zip(weights) {
        let rgba = vertex.color.to_array();
        for channel in 0..4 {
            color[channel] += f32::from(rgba[channel]) * weight / 255.0;
        }
    }
    color
}

fn interpolate_uv(vertices: [egui::epaint::Vertex; 3], weights: [f32; 3]) -> Pos2 {
    let mut uv = Vec2::ZERO;
    for (vertex, weight) in vertices.iter().zip(weights) {
        uv += vertex.uv.to_vec2() * weight;
    }
    uv.to_pos2()
}

fn sample_texture(texture: &Texture, uv: Pos2) -> [f32; 4] {
    let x = (uv.x * texture.width as f32)
        .floor()
        .clamp(0.0, texture.width as f32 - 1.0);
    let y = (uv.y * texture.height as f32)
        .floor()
        .clamp(0.0, texture.height as f32 - 1.0);
    let color = texture.pixels[y as usize * texture.width + x as usize].to_array();
    [
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
        f32::from(color[3]) / 255.0,
    ]
}

fn blend(destination: &mut [f32; 4], source: [f32; 4]) {
    let alpha = source[3];
    for channel in 0..4 {
        destination[channel] = source[channel] + destination[channel] * (1.0 - alpha);
    }
}

fn pixel_rgba(pixel: [f32; 4]) -> [u8; 4] {
    let alpha = pixel[3].clamp(0.0, 1.0);
    let scale = if alpha > 0.0 { 1.0 / alpha } else { 0.0 };
    [
        (pixel[0] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (pixel[1] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (pixel[2] * scale * 255.0).round().clamp(0.0, 255.0) as u8,
        (alpha * 255.0).round() as u8,
    ]
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/scene-320x240.rgba")
}

fn grid_golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/grid-320x240.rgba")
}

fn toolbar_golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/toolbar-2400x120.rgba")
}

#[cfg(test)]
mod tests {
    use super::{
        golden_path, grid_golden_path, render_golden, render_grid_golden, render_toolbar_golden,
        toolbar_golden_path,
    };

    #[test]
    fn golden_render_is_deterministic() {
        assert_eq!(render_golden(), render_golden());
        assert_eq!(render_grid_golden(), render_grid_golden());
        assert_eq!(render_toolbar_golden(), render_toolbar_golden());
    }

    #[test]
    fn golden_scene_matches_snapshot() {
        check_golden(&render_golden(), golden_path(), super::WIDTH);
    }

    #[test]
    fn golden_grid_matches_snapshot() {
        check_golden(&render_grid_golden(), grid_golden_path(), super::WIDTH);
    }

    #[test]
    fn golden_toolbar_matches_snapshot() {
        check_golden(&render_toolbar_golden(), toolbar_golden_path(), 2400);
    }

    fn check_golden(image: &[u8], path: std::path::PathBuf, width: usize) {
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("golden dir");
            }
            std::fs::write(&path, image).expect("write golden");
            return;
        }
        let expected = std::fs::read(&path).expect("read golden");
        assert_snapshot(image, &expected, width);
    }

    fn assert_snapshot(actual: &[u8], expected: &[u8], width: usize) {
        assert_eq!(actual.len(), expected.len(), "golden size mismatch");
        let Some((index, left, right)) = first_difference(actual, expected) else {
            return;
        };
        panic_mismatch(index, left, right, width);
    }

    fn first_difference(actual: &[u8], expected: &[u8]) -> Option<(usize, u8, u8)> {
        actual
            .iter()
            .zip(expected.iter())
            .enumerate()
            .find(|(_, (left, right))| left != right)
            .map(|(index, (left, right))| (index, *left, *right))
    }

    fn panic_mismatch(index: usize, left: u8, right: u8, width: usize) -> ! {
        let pixel = index / 4;
        let x = pixel % width;
        let y = pixel / width;
        panic!(
            "golden mismatch at pixel ({x}, {y}) channel {}: {left} != {right}",
            index % 4
        );
    }
}
