//! Canvas rendering for components and placement ghosts.

use crate::library;
use crate::model::{
    ComponentDefinition, Document, GridPos, Mirror, NetClass, Rotation, WireId, body_size,
    pin_direction, world_pin,
};
use crate::selection::Selection;
use eframe::egui;

const SELECTED_STROKE: egui::Color32 = egui::Color32::from_rgb(0, 200, 220);
const SELECTED_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 22, 24, 28);
const SYMBOL_STROKE: egui::Color32 = egui::Color32::from_rgb(190, 198, 210);
const PIN_COLOR: egui::Color32 = egui::Color32::from_rgb(215, 220, 230);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(200, 205, 215);
const JUNCTION_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 235, 245);
const GHOST_OK: egui::Color32 = egui::Color32::from_rgb(60, 210, 130);
const GHOST_BAD: egui::Color32 = egui::Color32::from_rgb(230, 80, 80);
const PREVIEW_BAD: egui::Color32 = egui::Color32::from_rgb(240, 90, 90);

#[derive(Clone, Copy)]
pub(crate) struct WireStyle {
    color: egui::Color32,
    width: f32,
    dash: Option<(f32, f32)>,
    flow_phase: Option<f32>,
}

/// Immutable scene used by the canvas renderer.
#[derive(Clone, Copy)]
pub struct Scene<'a> {
    pub document: &'a Document,
    pub library: &'a [ComponentDefinition],
    pub selection: &'a Selection,
    pub hover_wire: Option<WireId>,
    pub placing: Option<&'a ComponentDefinition>,
    pub rotation: Rotation,
    pub mirror: Mirror,
    pub wire_preview: &'a [GridPos],
    pub wire_class: NetClass,
    pub selection_box: Option<(GridPos, GridPos)>,
    pub minimap: bool,
    pub flow_phase: Option<f32>,
}

/// Draws all wires, components, and the placement ghost.
pub fn draw(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: Scene<'_>,
    hover: Option<GridPos>,
) {
    draw_wire_layer(painter, rect, camera, &scene);
    draw_junctions(painter, rect, camera, scene.document);
    draw_components(painter, rect, camera, &scene, hover);
    draw_wire_handles(painter, rect, camera, &scene);
    draw_wire_preview(painter, rect, camera, &scene, hover);
    draw_ghost_layer(painter, rect, camera, &scene, hover);
    draw_selection_box(painter, rect, camera, scene.selection_box);
    if scene.minimap {
        draw_minimap(painter, rect, camera, &scene);
    }
}

fn draw_wire_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    draw_wires(
        painter,
        rect,
        camera,
        scene.document,
        scene.selection,
        scene.hover_wire,
        scene.flow_phase,
    );
}

fn draw_ghost_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
    hover: Option<GridPos>,
) {
    let Some(definition) = scene.placing else {
        return;
    };
    let Some(hover) = hover else {
        return;
    };
    draw_ghost(painter, rect, camera, scene, definition, hover);
}

fn draw_selection_box(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    corners: Option<(GridPos, GridPos)>,
) {
    let Some((start, end)) = corners else {
        return;
    };
    let box_rect = screen_box(rect, camera, start, end);
    let fill = egui::Color32::from_rgba_unmultiplied(0, 200, 220, 28);
    painter.rect_filled(box_rect, 0.0, fill);
    painter.rect_stroke(
        box_rect,
        0.0,
        egui::Stroke::new(1.0, SELECTED_STROKE),
        egui::StrokeKind::Middle,
    );
}

pub fn placement_position(
    definition: &ComponentDefinition,
    hover: GridPos,
    rotation: Rotation,
) -> GridPos {
    let size = body_size(definition, rotation);
    hover.subtract(GridPos::new(size.width / 2, size.height / 2))
}

mod components;
mod minimap;
#[cfg(test)]
mod tests;
mod wires;

pub(crate) use components::*;
pub(crate) use minimap::*;
pub(crate) use wires::*;
