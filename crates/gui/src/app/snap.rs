use super::*;
use crate::model::world_pin;
use crate::scene::{SnapKind, SnapMarker};

const SNAP_RADIUS_PX: f32 = 8.0;
const MIN_SNAP_RADIUS: f32 = 0.25;
const MAX_SNAP_RADIUS: f32 = 0.75;

impl GuiApp {
    /// Returns the snapped cursor point and its source.
    pub(crate) fn snap_world(&self, world: egui::Pos2) -> SnapMarker {
        let (grid, grid_enabled) = self.grid_point(world);
        if self.snap_objects {
            if let Some(marker) = self.object_snap(world) {
                return marker;
            }
        }
        if !grid_enabled {
            return SnapMarker {
                point: grid,
                kind: SnapKind::Off,
            };
        }
        SnapMarker {
            point: grid,
            kind: SnapKind::Grid,
        }
    }

    fn grid_point(&self, world: egui::Pos2) -> (GridPos, bool) {
        if self.grid_snap_enabled() {
            return (world_to_grid_step(world, self.grid_step), true);
        }
        (world_to_grid(world), false)
    }

    pub(crate) fn grid_snap_enabled(&self) -> bool {
        match self.snap_mode {
            SnapMode::Always => true,
            SnapMode::WithGrid => self.grid_visible,
            SnapMode::Never => false,
        }
    }

    fn object_snap(&self, world: egui::Pos2) -> Option<SnapMarker> {
        let radius = self.snap_radius();
        let mut best = None;
        self.snap_pins(world, radius, &mut best);
        self.snap_wires(world, radius, &mut best);
        best.map(|(_, marker)| marker)
    }

    fn snap_radius(&self) -> f32 {
        let scale = self.camera.scale();
        if scale <= 0.0 {
            return MAX_SNAP_RADIUS;
        }
        (SNAP_RADIUS_PX / scale).clamp(MIN_SNAP_RADIUS, MAX_SNAP_RADIUS)
    }

    fn snap_pins(&self, world: egui::Pos2, radius: f32, best: &mut Option<(f32, SnapMarker)>) {
        for instance in self.document.components() {
            let Some(definition) = self
                .library
                .iter()
                .find(|def| def.id == instance.definition)
            else {
                continue;
            };
            for pin in definition.pins {
                let point = world_pin(instance, definition, pin);
                self.consider(point, SnapKind::Pin, world, radius, best);
            }
        }
    }

    fn snap_wires(&self, world: egui::Pos2, radius: f32, best: &mut Option<(f32, SnapMarker)>) {
        for point in self.document.junctions() {
            self.consider(point, SnapKind::Junction, world, radius, best);
        }
        for wire in self.document.wires() {
            for point in &wire.points {
                self.consider(*point, SnapKind::Wire, world, radius, best);
            }
        }
    }

    fn consider(
        &self,
        point: GridPos,
        kind: SnapKind,
        world: egui::Pos2,
        radius: f32,
        best: &mut Option<(f32, SnapMarker)>,
    ) {
        let distance = snap_distance(point, world);
        if distance > radius {
            return;
        }
        if best
            .as_ref()
            .is_some_and(|(current, _)| *current <= distance)
        {
            return;
        }
        *best = Some((distance, SnapMarker { point, kind }));
    }
}

fn snap_distance(point: GridPos, world: egui::Pos2) -> f32 {
    let dx = point.x as f32 - world.x;
    let dy = point.y as f32 - world.y;
    (dx * dx + dy * dy).sqrt()
}

/// Rounds a world position to the nearest multiple of a logical grid step.
#[must_use]
pub(crate) fn world_to_grid_step(position: egui::Pos2, step: GridStep) -> GridPos {
    let step = step.value() as f32;
    let x = (position.x / step).round() as i32 * step as i32;
    let y = (position.y / step).round() as i32 * step as i32;
    GridPos::new(x, y)
}
