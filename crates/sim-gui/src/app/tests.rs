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
fn additive_click_collects_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.select_at(eframe::egui::pos2(1.0, 1.0), false);
    assert_eq!(app.selection.single_component(), Some(first));
    app.select_at(eframe::egui::pos2(7.0, 1.0), true);
    assert_eq!(app.selection.components().len(), 2);
}

#[test]
fn drag_offsets_keep_relative_positions() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let second = document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.apply([first, second], [], false);
    let offsets = app.drag_offsets(GridPos::new(1, 0));
    assert_eq!(
        offsets,
        vec![(first, GridPos::new(-1, 0)), (second, GridPos::new(5, 0))]
    );
}

#[test]
fn selection_box_selects_components_and_wires() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let wire = sample_wire(&mut document, library);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.apply_box(GridPos::new(-1, -1), GridPos::new(10, 3), false);
    assert_eq!(app.selection.components().len(), 2);
    assert!(app.selection.contains_wire(wire));
}

#[test]
fn copy_and_paste_selects_new_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    let first = document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        ..GuiApp::default()
    };
    app.selection.select_component(first);
    app.copy_selection();
    app.paste_selection();
    assert_eq!(app.document.components().len(), 2);
    assert_eq!(app.selection.components().len(), 1);
    assert_ne!(app.selection.single_component(), Some(first));
}

fn context_click(
    world: eframe::egui::Pos2,
    screen: eframe::egui::Pos2,
) -> crate::canvas::CanvasState {
    crate::canvas::CanvasState {
        context_click: Some(crate::canvas::ContextClick { screen, world }),
        ..Default::default()
    }
}

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

#[test]
fn recent_components_are_bounded_and_deduplicated() {
    let definitions = crate::library::builtin();
    let mut app = GuiApp::default();
    for definition in definitions.iter().take(6) {
        app.remember_component(definition);
    }
    assert_eq!(app.recent_components.len(), 5);
    assert_eq!(app.recent_components[0].id, definitions[5].id);
    app.remember_component(&definitions[0]);
    assert_eq!(app.recent_components[0].id, definitions[0].id);
}

#[test]
fn content_bounds_cover_components() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let bounds = content_bounds(&document, library, None).expect("bounds");
    assert_eq!(bounds, (GridPos::new(0, 0), GridPos::new(10, 2)));
}

#[test]
fn delete_confirmation_for_wired_or_large_selection() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    assert!(!app.needs_delete_confirmation());
    sample_wire(&mut app.document, app.library);
    assert!(app.needs_delete_confirmation());
    let large: Vec<_> = (0..6u32).map(ComponentId).collect();
    app.selection.apply(large, [], false);
    assert!(app.needs_delete_confirmation());
}

#[test]
fn wire_tool_starts_at_pin() {
    let mut app = wire_app(Vec::new());
    app.handle_canvas(click_state(5.0, 1.0));
    assert_eq!(app.wire_points, vec![GridPos::new(4, 1)]);
}

#[test]
fn wire_tool_extends_to_valid_point() {
    let mut app = wire_app(vec![GridPos::new(4, 1)]);
    app.handle_canvas(click_state(8.0, 1.0));
    assert_eq!(
        app.wire_points,
        vec![GridPos::new(4, 1), GridPos::new(8, 1)]
    );
}

#[test]
fn wire_tool_finishes_at_second_pin() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        tool: crate::app::Tool::Wire,
        wire_points: vec![GridPos::new(4, 1)],
        ..GuiApp::default()
    };
    app.handle_canvas(click_state(6.0, 1.0));
    assert_eq!(app.document.wires().len(), 1);
    assert!(app.wire_points.is_empty());
}

#[test]
fn wire_class_is_applied_to_new_wire() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    document.add(resistor, GridPos::new(6, 0), Rotation::Deg0, Mirror::None);
    let mut app = GuiApp {
        library,
        document,
        tool: crate::app::Tool::Wire,
        wire_class: NetClass::Power,
        wire_points: vec![GridPos::new(4, 1)],
        ..GuiApp::default()
    };
    app.handle_canvas(click_state(6.0, 1.0));
    assert_eq!(app.document.wires()[0].class, NetClass::Power);
}

#[test]
fn selected_wire_class_can_be_changed() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_class(NetClass::Analog);
    assert_eq!(app.document.wires()[0].class, NetClass::Analog);
    app.undo();
    assert_eq!(app.document.wires()[0].class, NetClass::Digital);
}

#[test]
fn selected_wire_color_can_be_overridden() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_color_override(Some([1, 2, 3]));
    assert_eq!(app.document.wires()[0].color_override, Some([1, 2, 3]));
    app.undo();
    assert_eq!(app.document.wires()[0].color_override, None);
}

#[test]
fn selected_wire_label_can_be_set() {
    let (mut app, _, _) = context_app();
    let wire = sample_wire(&mut app.document, app.library);
    app.selection.apply([], [wire], false);
    app.set_wire_label(Some("LED_A".to_string()));
    assert_eq!(app.document.wires()[0].label.as_deref(), Some("LED_A"));
    app.undo();
    assert_eq!(app.document.wires()[0].label, None);
}

#[test]
fn place_at_commits_definition() {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut app = GuiApp {
        library,
        placing: Some(resistor),
        drag_place: true,
        ..GuiApp::default()
    };
    app.place_at(eframe::egui::pos2(0.0, 0.0));
    assert_eq!(app.document.components().len(), 1);
    assert!(app.placing.is_none());
    assert!(!app.drag_place);
}

#[test]
fn nudge_selection_moves_component() {
    let (mut app, id) = nudged_app();
    app.nudge_selection(1, 0);
    assert_component_position(&app, id, GridPos::new(1, 0));
    app.nudge_selection(0, 2);
    assert_component_position(&app, id, GridPos::new(1, 2));
}

#[test]
fn analog_plot_maps_cycle_and_value() {
    let plot = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(10.0, 20.0),
        eframe::egui::vec2(100.0, 50.0),
    );
    assert!((super::cycle_to_x(plot, (0, 100), 50) - 60.0).abs() < 0.01);
    assert_eq!(super::cycle_at(plot, (0, 100), 60.0), 50);
    assert!((super::value_to_y(plot, (0.0, 10.0), 10.0) - 20.0).abs() < 0.01);
}

#[test]
fn analog_graph_renders_in_egui_context() {
    let (probes, mut view) = analog_fixture();
    let mut output = render_analog(&probes, &mut view);
    output.textures_delta.clear();
    assert!(!output.shapes.is_empty());
}

fn analog_fixture() -> (crate::probe::ProbeSet, crate::probe::PlotView) {
    let mut probes = crate::probe::ProbeSet::default();
    assert!(probes.add(analog_spec()));
    probes.sample_from(&analog_provider(), 100);
    (probes, crate::probe::PlotView::default())
}

fn analog_spec() -> crate::probe::ProbeSpec {
    crate::probe::ProbeSpec {
        label: "PB5".to_string(),
        unit: "V",
        color: eframe::egui::Color32::WHITE,
        source: crate::probe::ProbeSource::Pin {
            component: ComponentId(0),
            pin: "PB5",
        },
        sampling: crate::probe::Sampling::default(),
    }
}

fn analog_provider() -> crate::probe::SampleProvider {
    crate::probe::SampleProvider {
        sample: crate::sim::Sample {
            cycle: 100,
            pb5_high: true,
            pb5_millivolts: 5000,
            led_on: true,
            led_microamps: 14_000,
        },
    }
}

fn render_analog(
    probes: &crate::probe::ProbeSet,
    view: &mut crate::probe::PlotView,
) -> eframe::egui::FullOutput {
    eframe::egui::Context::default().run_ui(eframe::egui::RawInput::default(), |ui| {
        super::show_analog_graph(ui, probes, view, crate::sim::Status::default());
    })
}

fn nudged_app() -> (GuiApp, ComponentId) {
    let library = crate::library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut app = GuiApp {
        library,
        ..GuiApp::default()
    };
    let id = app
        .document
        .add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    app.selection.select_component(id);
    (app, id)
}

fn assert_component_position(app: &GuiApp, id: ComponentId, expected: GridPos) {
    assert_eq!(
        app.document.component(id).expect("component").position,
        expected
    );
}

#[test]
fn flow_phase_is_optional() {
    assert_eq!(super::flow_phase(false, 1.0), None);
    let phase = super::flow_phase(true, 1.0).expect("phase");
    assert!((0.0..1000.0).contains(&phase));
}

#[test]
fn every_tool_has_an_icon() {
    for tool in super::Tool::ALL {
        assert!(!tool.icon().is_empty());
        assert!(!tool.label().is_empty());
    }
}

#[test]
fn context_click_selects_target() {
    let (mut app, first, second) = context_app();
    app.selection.select_component(second);
    app.open_context_menu(&context_click(
        eframe::egui::pos2(1.0, 1.0),
        eframe::egui::pos2(10.0, 10.0),
    ));
    assert_eq!(app.selection.single_component(), Some(first));
    assert_eq!(app.context_menu, Some(eframe::egui::pos2(10.0, 10.0)));
}

#[test]
fn context_click_empty_clears() {
    let (mut app, first, _) = context_app();
    app.selection.select_component(first);
    app.open_context_menu(&context_click(
        eframe::egui::pos2(30.0, 30.0),
        eframe::egui::pos2(20.0, 20.0),
    ));
    assert!(app.selection.is_empty());
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

#[test]
fn l_route_adds_one_corner() {
    let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), true);
    assert_eq!(route, vec![GridPos::new(4, 0), GridPos::new(4, 2)]);
    let route = l_route(GridPos::new(0, 0), GridPos::new(4, 2), false);
    assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(4, 2)]);
}

#[test]
fn l_route_handles_aligned_and_same_points() {
    assert_eq!(
        l_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
        vec![GridPos::new(4, 0)]
    );
    assert!(l_route(GridPos::new(0, 0), GridPos::new(0, 0), true).is_empty());
}

#[test]
fn z_route_adds_two_corners() {
    let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), true);
    assert_eq!(
        route,
        vec![GridPos::new(3, 0), GridPos::new(3, 4), GridPos::new(6, 4)]
    );
    let route = z_route(GridPos::new(0, 0), GridPos::new(6, 4), false);
    assert_eq!(
        route,
        vec![GridPos::new(0, 2), GridPos::new(6, 2), GridPos::new(6, 4)]
    );
}

#[test]
fn z_route_simplifies_short_and_aligned() {
    let route = z_route(GridPos::new(0, 0), GridPos::new(1, 2), true);
    assert_eq!(route, vec![GridPos::new(0, 2), GridPos::new(1, 2)]);
    assert_eq!(
        z_route(GridPos::new(0, 0), GridPos::new(4, 0), true),
        vec![GridPos::new(4, 0)]
    );
    assert!(z_route(GridPos::new(0, 0), GridPos::new(0, 0), false).is_empty());
}

#[test]
fn cycle_at_x_maps_plot_edges() {
    let plot = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(100.0, 40.0),
    );
    assert_eq!(cycle_at_x(plot, (10, 110), 0.0), 10);
    assert_eq!(cycle_at_x(plot, (10, 110), 100.0), 110);
    assert_eq!(cycle_at_x(plot, (10, 110), 50.0), 60);
}

#[test]
fn move_segment_shifts_both_endpoints() {
    let mut points = vec![
        GridPos::new(0, 0),
        GridPos::new(0, 2),
        GridPos::new(4, 2),
        GridPos::new(4, 4),
    ];
    assert!(move_segment(&mut points, 1, GridPos::new(0, 3)));
    assert_eq!(points[1], GridPos::new(0, 3));
    assert_eq!(points[2], GridPos::new(4, 3));
}
