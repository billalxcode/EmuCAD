//! GUI application shell, library sheet, inspector, and canvas integration.

mod context_ui;
mod editor;
mod graph_ui;
mod inspector;
mod legend_ui;
mod library_ui;
mod palette_ui;
mod project;
mod scene_view;
mod snap;
pub(crate) use snap::world_to_grid_step;
mod status;
mod support;
#[cfg(test)]
mod tests;
mod uart_ui;
mod wire_ui;
pub(crate) use context_ui::*;
pub(crate) use graph_ui::*;
pub(crate) use legend_ui::*;
pub(crate) use library_ui::*;
pub(crate) use palette_ui::*;
pub(crate) use scene_view::*;
pub(crate) use support::*;
pub(crate) use uart_ui::*;
pub(crate) use wire_ui::*;
mod toolbar;
mod view;
#[cfg(test)]
pub(crate) use view::fit_margin;
mod windows;

use crate::analysis::{AnalysisKind, AnalysisState, SimulationDomain};
use crate::canvas::{self, GridCamera};
pub(crate) use crate::canvas::{CrosshairMode, GridStep, SnapMode, Units};
use crate::clipboard::{self, Clipboard};
use crate::config;
use crate::error::{GuiError, GuiResult};
use crate::export;
use crate::history::History;
use crate::layout::{self, Align, Distribute};
use crate::library;
use crate::log::Log;
use crate::model::{
    Category, ComponentDefinition, ComponentId, ComponentInstance, Document, GridPos, GridRect,
    McuPinMode, Mirror, NetClass, Rotation, WireId,
};
use crate::palette::{self, Command};
use crate::persist;
use crate::probe::{self, PlotView, ProbeSet, ProbeSource, ProbeSpec, SampleProvider, Sampling};
use crate::provider::{BuiltinComponents, ComponentProvider};
use crate::scene::{self, Scene};
use crate::selection::Selection;
use crate::sim::{self, Event, Sample, SimHandle, Status, Waveform};
use crate::uart::UartLog;
use eframe::egui;
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

const AUTOSAVE_DELAY: Duration = Duration::from_secs(2);
const WATCH_INTERVAL: Duration = Duration::from_secs(1);
const TOAST_DURATION: Duration = Duration::from_secs(3);
const PB5_COLOR: egui::Color32 = egui::Color32::from_rgb(75, 117, 255);
const LED_COLOR: egui::Color32 = egui::Color32::from_rgb(14, 175, 98);
const CONTEXT_COLORS: [(&str, [u8; 3]); 6] = [
    ("Red", [225, 85, 85]),
    ("Green", [85, 205, 125]),
    ("Blue", [90, 155, 240]),
    ("Cyan", [80, 215, 225]),
    ("Yellow", [230, 210, 95]),
    ("White", [230, 235, 245]),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Tool {
    #[default]
    Select,
    Place,
    Wire,
    Probe,
    Zoom,
}

impl Tool {
    const ALL: [Self; 5] = [
        Self::Select,
        Self::Place,
        Self::Wire,
        Self::Probe,
        Self::Zoom,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Place => "Place",
            Self::Wire => "Wire",
            Self::Probe => "Probe",
            Self::Zoom => "Zoom",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Self::Select => "*",
            Self::Place => "+",
            Self::Wire => "~",
            Self::Probe => "?",
            Self::Zoom => "Z",
        }
    }
}

/// Area used by a fit command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FitTarget {
    All,
    Objects,
    Selection,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InspectorAction {
    Rotate,
    Mirror,
    Delete,
    SetClass(NetClass),
    SetColor(Option<[u8; 3]>),
    SetLabel(Option<String>),
    SetParameter { name: &'static str, value: f64 },
    SetPinMode { pin: u8, mode: McuPinMode },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SimAction {
    Run,
    Pause,
    Step,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConflictAction {
    Reload,
    SaveMine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ContextAction {
    Copy,
    Paste,
    Duplicate,
    Delete,
    Rotate,
    Mirror,
    Properties,
    Graph,
    ProbePb5,
    ProbeLed,
    SetClass(NetClass),
    SetColor(Option<[u8; 3]>),
    SetLabel(Option<String>),
    SelectTool,
    WireTool,
}

pub(crate) struct ContextNetState {
    class: Option<NetClass>,
    has_override: bool,
    label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SegmentDrag {
    wire: WireId,
    index: usize,
}

#[derive(Clone, Debug)]
struct DragState {
    offsets: Vec<(ComponentId, GridPos)>,
}

#[derive(Clone, Copy)]
pub(crate) struct SceneRefs<'a> {
    document: &'a Document,
    library: &'a [ComponentDefinition],
    selection: &'a Selection,
    wire_preview: &'a [GridPos],
}

#[derive(Clone, Copy)]
pub(crate) struct SceneState {
    hover_wire: Option<WireId>,
    placing: Option<&'static ComponentDefinition>,
    rotation: Rotation,
    mirror: Mirror,
    wire_class: NetClass,
    selection_box: Option<(GridPos, GridPos)>,
    snap: Option<crate::scene::SnapMarker>,
    crosshair: CrosshairMode,
    minimap: bool,
    flow_phase: Option<f32>,
}

pub(crate) struct LibraryState {
    search: String,
    category: Option<Category>,
    chosen: Option<&'static ComponentDefinition>,
    hovered: Option<&'static ComponentDefinition>,
    dragged: Option<&'static ComponentDefinition>,
}

#[derive(Default)]
pub struct GuiApp {
    camera: GridCamera,
    canvas_rect: Option<egui::Rect>,
    grid_visible: bool,
    grid_step: GridStep,
    grid_style: canvas::GridStyle,
    grid_axes: bool,
    grid_rulers: bool,
    grid_line_width: f32,
    grid_min_spacing: f32,
    fast_grid_1: GridStep,
    fast_grid_2: GridStep,
    grid_settings_open: bool,
    crosshair: CrosshairMode,
    units: Units,
    local_origin: GridPos,
    snap_mode: SnapMode,
    snap_objects: bool,
    minimap_open: bool,
    flow_animation: bool,
    light_theme: bool,
    tool: Tool,
    pointer_world: Option<egui::Pos2>,
    initialized: bool,
    canvas_focused: bool,
    sample_every: u64,
    analysis_kind: AnalysisKind,
    simulation_domain: SimulationDomain,
    simulation_open: bool,
    library: &'static [ComponentDefinition],
    recent_components: Vec<&'static ComponentDefinition>,
    document: Document,
    selection: Selection,
    clipboard: Clipboard,
    context_menu: Option<egui::Pos2>,
    pending_delete: bool,
    toast_until: Option<Instant>,
    last_toast: String,
    box_start: Option<GridPos>,
    box_current: Option<GridPos>,
    box_additive: bool,
    dragging: Option<DragState>,
    dragging_segment: Option<SegmentDrag>,
    placing: Option<&'static ComponentDefinition>,
    drag_place: bool,
    placement_rotation: Rotation,
    placement_mirror: Mirror,
    library_open: bool,
    inspector_open: bool,
    legend_open: bool,
    palette: palette::State,
    search: String,
    category: Option<Category>,
    wire_points: Vec<GridPos>,
    wire_class: NetClass,
    current_path: Option<PathBuf>,
    message: String,
    log: Log,
    log_open: bool,
    history: History,
    drag_history_pending: bool,
    sim: Option<SimHandle>,
    firmware_words: Vec<u16>,
    sim_status: Status,
    waveform: Waveform,
    waveform_open: bool,
    analog_open: bool,
    probes: ProbeSet,
    plot_view: PlotView,
    transient: Option<crate::transient::TransientTrace>,
    transient_settings: crate::transient::TransientSettings,
    transient_selected: BTreeSet<usize>,
    transient_selected_currents: BTreeSet<usize>,
    transient_open: bool,
    analog: Option<crate::analog::AnalogHandle>,
    mixed_worker: Option<crate::mixed_worker::MixedHandle>,
    mixed_trace: Option<crate::mixed_worker::MixedTrace>,
    mixed_request: Option<u64>,
    analog_request: Option<u64>,
    analog_busy: bool,
    analog_paused: bool,
    analog_dc: Option<crate::analog::DcResultView>,
    uart: UartLog,
    uart_input: String,
    uart_open: bool,
    config: config::AppConfig,
    config_path: Option<PathBuf>,
    png_result: Arc<Mutex<Option<Result<(), String>>>>,
    dirty: bool,
    last_edit: Option<Instant>,
    file_stamp: Option<FileStamp>,
    watch_at: Option<Instant>,
    conflict: bool,
}

impl GuiApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        crate::design::configure(context);
        let (config_path, config) = load_config();
        let mut app = Self::base(config_path, config);
        app.sim = Some(SimHandle::spawn());
        app.analog = Some(crate::analog::AnalogHandle::spawn());
        app.mixed_worker = Some(crate::mixed_worker::MixedHandle::spawn());
        app.add_pb5_probe();
        app.add_led_probe();
        app
    }

    fn base(config_path: Option<PathBuf>, config: config::AppConfig) -> Self {
        Self {
            library: BuiltinComponents.definitions(),
            document: Document::new(),
            grid_visible: true,
            grid_rulers: true,
            grid_line_width: 1.0,
            grid_min_spacing: 10.0,
            fast_grid_1: GridStep::ALL[0],
            fast_grid_2: GridStep::ALL[3],
            snap_objects: true,
            inspector_open: true,
            config_path,
            config,
            ..Self::default()
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.palette_shortcut(ui);
        self.poll_sim();
        self.poll_analog();
        self.poll_mixed();
        self.watch_file(ui.ctx());
        self.autosave(ui.ctx());
        self.poll_png();
        self.update_toast();
        self.toolbar(ui);
        self.status_bar(ui);
        let state = self.canvas(ui);
        self.pointer_world = state.pointer_world;
        self.canvas_rect = Some(state.rect);
        self.handle_canvas(state);
        self.finish_drag_place(ui.ctx());
        self.handle_keys(ui);
        self.windows(ui);
    }

    fn on_exit(&mut self) {
        let _ = self.flush_autosave();
    }
}
