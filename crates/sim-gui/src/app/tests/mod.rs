use super::{
    AUTOSAVE_DELAY, ConflictAction, GuiApp, WATCH_INTERVAL, content_bounds, cycle_at_x, file_stamp,
    l_route, move_segment, watch_remaining, z_route,
};
use crate::canvas::GridCamera;
use crate::model::{
    ComponentDefinition, ComponentId, Document, GridPos, Mirror, NetClass, Rotation, WireId,
};
use crate::persist;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("emucad-app-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn dirty_app(path: &Path) -> GuiApp {
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    GuiApp {
        library: crate::library::builtin(),
        document,
        current_path: Some(path.to_path_buf()),
        dirty: true,
        last_edit: Some(Instant::now()),
        ..GuiApp::default()
    }
}

fn sample_wire(document: &mut Document, library: &[ComponentDefinition]) -> WireId {
    document
        .add_wire(
            library,
            vec![GridPos::new(4, 1), GridPos::new(6, 1)],
            NetClass::Digital,
        )
        .expect("wire")
}

fn wire_app(points: Vec<GridPos>) -> GuiApp {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    GuiApp {
        library,
        document,
        tool: crate::app::Tool::Wire,
        wire_points: points,
        ..GuiApp::default()
    }
}

fn click_state(x: f32, y: f32) -> crate::canvas::CanvasState {
    crate::canvas::CanvasState {
        clicked: true,
        click_world: Some(eframe::egui::pos2(x, y)),
        ..Default::default()
    }
}

mod analog;
mod context;
mod editing;
mod geometry;
mod graph;
mod library;
mod persistence;
mod wiring;

use super::*;

fn context_app() -> (GuiApp, ComponentId, ComponentId) {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    (
        GuiApp {
            library,
            document,
            ..GuiApp::default()
        },
        first,
        second,
    )
}
