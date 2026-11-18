use super::*;

pub(crate) fn file_stamp(path: &Path) -> Option<FileStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    })
}

pub(crate) fn watch_remaining(last: Option<Instant>, now: Instant) -> Duration {
    last.map_or(Duration::ZERO, |last| {
        WATCH_INTERVAL.saturating_sub(now.saturating_duration_since(last))
    })
}

pub(crate) fn show_log_contents(ui: &mut egui::Ui, log: &Log, clear: &mut bool) {
    if ui.button("Clear").clicked() {
        *clear = true;
    }
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for entry in log.entries() {
                ui.monospace(entry);
            }
        });
}

pub(crate) fn delete_dialog(ui: &mut egui::Ui, count: usize, action: &mut Option<bool>) {
    ui.set_width(320.0);
    ui.heading("Delete selection?");
    ui.label(format!("{count} items will be removed."));
    ui.horizontal(|ui| {
        if ui.button("Delete").clicked() {
            *action = Some(true);
        }
        if ui.button("Cancel").clicked() {
            *action = Some(false);
        }
    });
}

pub(crate) fn conflict_contents(
    ui: &mut egui::Ui,
    path: &Path,
    action: &mut Option<ConflictAction>,
) {
    ui.set_width(380.0);
    ui.heading("File changed on disk");
    ui.label("The project was modified outside EmuCAD.");
    ui.label(path.display().to_string());
    ui.label("Reload discards local edits. Save Mine overwrites the file.");
    ui.horizontal(|ui| {
        if ui.button("Reload").clicked() {
            *action = Some(ConflictAction::Reload);
        }
        if ui.button("Save Mine").clicked() {
            *action = Some(ConflictAction::SaveMine);
        }
    });
}

pub(crate) fn load_config() -> (Option<PathBuf>, config::AppConfig) {
    let Some(path) = config::path() else {
        return (None, config::AppConfig::default());
    };
    let config = config::load(&path);
    (Some(path), config)
}

pub(crate) fn select_save_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("EmuCAD project", &["json"])
        .set_file_name("project.emu.json")
        .save_file()
}

pub(crate) fn select_export_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("PNG image", &["png"])
        .set_file_name("schematic.png")
        .save_file()
}

pub(crate) fn png_callback(
    screen: egui::Rect,
    path: PathBuf,
    canvas: egui::Rect,
    slot: Arc<Mutex<Option<Result<(), String>>>>,
) -> egui::ScreenshotCallback {
    egui::ScreenshotCallback::new(move |image| {
        let result = export::save_crop(&path, &image, canvas, screen);
        if let Ok(mut slot) = slot.lock() {
            *slot = Some(result);
        }
    })
}

pub(crate) fn sim_command(sim: &SimHandle, action: SimAction) -> Result<(), String> {
    match action {
        SimAction::Run => sim.run(),
        SimAction::Pause => sim.pause(),
        SimAction::Step => sim.step(),
        SimAction::Reset => sim.reset(),
    }
}

pub(crate) fn pick_hex() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Intel HEX", &["hex", "ihx"])
        .pick_file()
}

pub(crate) fn file_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || "firmware".to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
