//! `emucad-gui` native desktop entry point.

mod analog;
mod analysis;
mod app;
mod canvas;
mod clipboard;
mod config;
mod design;
mod error;
mod export;
#[cfg(test)]
mod golden;
mod history;
mod icons;
mod layout;
mod library;
mod log;
mod mixed_worker;
mod model;
mod netlist;
mod palette;
mod persist;
mod probe;
mod provider;
mod scene;
mod selection;
mod sim;
#[cfg(test)]
mod solver_tests;
mod spatial;
mod symbols;
mod transient;
mod uart;

use app::GuiApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "EmuCAD",
        options,
        Box::new(|context| Ok(Box::new(GuiApp::new(context)))),
    )
}
