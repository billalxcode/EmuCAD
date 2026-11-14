use super::*;

#[test]
fn open_flushes_dirty_document_and_updates_recent() {
    let dir = temp_dir("open-flush");
    let old = dir.join("old.emu.json");
    let new = dir.join("new.emu.json");
    let camera = GridCamera::default();
    persist::save(&old, &Document::new(), &camera, true).expect("old");
    persist::save(&new, &Document::new(), &camera, true).expect("new");
    let mut app = dirty_app(&old);
    app.open_path(new.clone());
    let text = std::fs::read_to_string(&old).expect("flushed");
    assert!(text.contains("resistor"));
    assert_eq!(app.current_path, Some(new.clone()));
    assert_eq!(app.config.recent_files.first(), Some(&new));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn open_is_blocked_for_unsaved_project() {
    let mut app = GuiApp {
        dirty: true,
        ..GuiApp::default()
    };
    assert!(!app.flush_autosave());
    assert!(app.message.contains("Save project"));
}

#[test]
fn autosave_waits_for_conflict_resolution() {
    let now = Instant::now();
    let app = GuiApp {
        conflict: true,
        dirty: true,
        current_path: Some(PathBuf::from("/tmp/project.emu.json")),
        last_edit: Some(now),
        ..GuiApp::default()
    };
    assert_eq!(app.autosave_remaining(now + AUTOSAVE_DELAY), None);
}

#[test]
fn watch_remaining_respects_interval() {
    let now = Instant::now();
    assert_eq!(watch_remaining(None, now), Duration::ZERO);
    assert_eq!(watch_remaining(Some(now), now), WATCH_INTERVAL);
    assert_eq!(
        watch_remaining(Some(now), now + WATCH_INTERVAL),
        Duration::ZERO
    );
}

#[test]
fn external_write_sets_conflict() {
    let dir = temp_dir("watch");
    let path = dir.join("project.emu.json");
    let camera = GridCamera::default();
    persist::save(&path, &Document::new(), &camera, true).expect("save");
    let stamp = file_stamp(&path).expect("stamp");
    let past = Instant::now().checked_sub(WATCH_INTERVAL).expect("clock");
    let mut app = GuiApp {
        current_path: Some(path.clone()),
        file_stamp: Some(stamp),
        watch_at: Some(past),
        ..GuiApp::default()
    };
    std::fs::write(&path, "external change").expect("write");
    app.watch_file(&eframe::egui::Context::default());
    assert!(app.conflict);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn autosave_refuses_external_change() {
    let dir = temp_dir("autosave-conflict");
    let path = dir.join("project.emu.json");
    persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
    let stamp = file_stamp(&path).expect("stamp");
    let mut app = dirty_app(&path);
    app.file_stamp = Some(stamp);
    std::fs::write(&path, "external change").expect("write");
    app.write_autosave(Instant::now());
    assert!(app.conflict);
    assert!(app.dirty);
    let text = std::fs::read_to_string(&path).expect("file");
    assert_eq!(text, "external change");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn manual_save_detects_external_change() {
    let dir = temp_dir("save-watch");
    let path = dir.join("project.emu.json");
    persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
    let stamp = file_stamp(&path).expect("stamp");
    let mut app = dirty_app(&path);
    app.file_stamp = Some(stamp);
    std::fs::write(&path, "external change").expect("write");
    app.save_project();
    assert!(app.conflict);
    let text = std::fs::read_to_string(&path).expect("file");
    assert_eq!(text, "external change");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn reload_replaces_document_and_clears_conflict() {
    let dir = temp_dir("reload");
    let path = dir.join("project.emu.json");
    persist::save(&path, &Document::new(), &GridCamera::default(), true).expect("save");
    let mut external = Document::new();
    let resistor = crate::library::find("resistor").expect("resistor");
    external.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    persist::save(&path, &external, &GridCamera::default(), true).expect("external");
    let mut app = GuiApp {
        library: crate::library::builtin(),
        current_path: Some(path.clone()),
        conflict: true,
        ..GuiApp::default()
    };
    app.reload_from_disk();
    assert!(!app.conflict);
    assert_eq!(app.document.components().len(), 1);
    assert_eq!(app.message, "Reloaded");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn save_mine_overwrites_external_file() {
    let dir = temp_dir("save-mine");
    let path = dir.join("project.emu.json");
    let camera = GridCamera::default();
    persist::save(&path, &Document::new(), &camera, true).expect("save");
    let mut app = dirty_app(&path);
    app.conflict = true;
    app.apply_conflict(ConflictAction::SaveMine);
    assert!(!app.conflict);
    let text = std::fs::read_to_string(&path).expect("saved");
    assert!(text.contains("resistor"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn autosave_requires_a_path_and_full_delay() {
    let now = Instant::now();
    let mut app = GuiApp {
        dirty: true,
        last_edit: Some(now),
        ..GuiApp::default()
    };
    assert_eq!(app.autosave_remaining(now), None);
    app.current_path = Some(PathBuf::from("/tmp/project.emu.json"));
    assert_eq!(app.autosave_remaining(now), Some(AUTOSAVE_DELAY));
    assert_eq!(
        app.autosave_remaining(now + AUTOSAVE_DELAY),
        Some(Duration::ZERO)
    );
    app.current_path = None;
}

#[test]
fn autosave_writes_and_clears_dirty() {
    let dir = std::env::temp_dir().join(format!("emucad-autosave-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("project.emu.json");
    let mut app = GuiApp {
        current_path: Some(path.clone()),
        dirty: true,
        last_edit: Some(Instant::now()),
        ..GuiApp::default()
    };
    app.write_autosave(Instant::now());
    assert!(!app.dirty);
    assert!(path.exists());
    assert_eq!(app.message, "Autosaved");
    let _ = std::fs::remove_dir_all(dir);
}
