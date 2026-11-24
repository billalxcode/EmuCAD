//! Canvas rendering for components and placement ghosts.

use crate::library;
use crate::model::{
    ComponentDefinition, Document, GridPos, Mirror, NetClass, Rotation, WireId, body_size,
    pin_direction, world_pin,
};
use crate::selection::Selection;
use eframe::egui;

pub(crate) fn palette(painter: &egui::Painter) -> crate::design::Palette {
    crate::design::palette_for(painter.ctx())
}

/// Source of the current snap point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapKind {
    Grid,
    Off,
    Pin,
    Wire,
    Junction,
}

impl SnapKind {
    /// Returns a short status label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Grid => "grid",
            Self::Off => "off",
            Self::Pin => "pin",
            Self::Wire => "wire",
            Self::Junction => "junction",
        }
    }
}

/// Snapped cursor position and its source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapMarker {
    pub point: GridPos,
    pub kind: SnapKind,
}

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
    pub snap: Option<SnapMarker>,
    pub minimap: bool,
    pub flow_phase: Option<f32>,
}

#[cfg(test)]
type SceneParts<'a> = (&'a Document, &'a [ComponentDefinition], &'a Selection);

impl<'a> Scene<'a> {
    /// Builds an empty scene with default interaction state.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(parts: SceneParts<'a>) -> Self {
        let (document, library, selection) = parts;
        Self {
            document,
            library,
            selection,
            hover_wire: None,
            placing: None,
            rotation: Rotation::Deg0,
            mirror: Mirror::None,
            wire_preview: &[],
            wire_class: NetClass::Digital,
            selection_box: None,
            snap: None,
            minimap: false,
            flow_phase: None,
        }
    }
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
    draw_dangling_layer(painter, rect, camera, &scene);
    draw_components(painter, rect, camera, &scene, hover);
    draw_wire_handles(painter, rect, camera, &scene);
    draw_wire_preview(painter, rect, camera, &scene, hover);
    draw_ghost_layer(painter, rect, camera, &scene, hover);
    draw_snap_marker(painter, rect, camera, scene.snap);
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

fn draw_dangling_layer(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    scene: &Scene<'_>,
) {
    for point in scene.document.dangling_endpoints(scene.library) {
        draw_dangling_marker(painter, rect, camera, point);
    }
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

fn draw_snap_marker(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &crate::canvas::GridCamera,
    marker: Option<SnapMarker>,
) {
    let Some(marker) = marker else {
        return;
    };
    let Some(color) = snap_color(marker.kind, &palette(painter)) else {
        return;
    };
    let point = egui::pos2(marker.point.x as f32, marker.point.y as f32);
    let screen = camera.world_to_screen(rect, point);
    painter.circle_stroke(screen, 5.0, egui::Stroke::new(1.5, color));
    painter.circle_filled(screen, 2.0, color);
}

fn snap_color(kind: SnapKind, colors: &crate::design::Palette) -> Option<egui::Color32> {
    match kind {
        SnapKind::Pin => Some(colors.accent),
        SnapKind::Wire => Some(colors.success),
        SnapKind::Junction => Some(colors.warning),
        SnapKind::Grid | SnapKind::Off => None,
    }
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
    let colors = palette(painter);
    painter.rect_filled(box_rect, 0.0, colors.selection);
    painter.rect_stroke(
        box_rect,
        0.0,
        egui::Stroke::new(1.0, colors.accent),
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
