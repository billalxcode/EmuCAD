//! Command palette catalog and state.

use crate::icons::IconKind;

/// One executable palette command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Save,
    Open,
    LoadHex,
    RunPause,
    Step,
    Reset,
    Undo,
    Redo,
    Copy,
    Paste,
    Duplicate,
    Delete,
    ToggleGrid,
    ToggleMinimap,
    ToggleLibrary,
    ToggleInspector,
    ToggleLegend,
    ToggleWaveform,
    ToggleUart,
    ToggleLog,
    ZoomReset,
    ZoomUndo,
    SelectTool,
    WireTool,
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterX,
    AlignCenterY,
    DistributeHorizontal,
    DistributeVertical,
}

impl Command {
    /// All commands shown by the palette.
    pub const ALL: [Self; 32] = [
        Self::Save,
        Self::Open,
        Self::LoadHex,
        Self::RunPause,
        Self::Step,
        Self::Reset,
        Self::Undo,
        Self::Redo,
        Self::Copy,
        Self::Paste,
        Self::Duplicate,
        Self::Delete,
        Self::ToggleGrid,
        Self::ToggleMinimap,
        Self::ToggleLibrary,
        Self::ToggleInspector,
        Self::ToggleLegend,
        Self::ToggleWaveform,
        Self::ToggleUart,
        Self::ToggleLog,
        Self::ZoomReset,
        Self::ZoomUndo,
        Self::SelectTool,
        Self::WireTool,
        Self::AlignLeft,
        Self::AlignRight,
        Self::AlignTop,
        Self::AlignBottom,
        Self::AlignCenterX,
        Self::AlignCenterY,
        Self::DistributeHorizontal,
        Self::DistributeVertical,
    ];

    /// Returns the display label for the current simulation state.
    #[must_use]
    pub fn label(self, running: bool) -> &'static str {
        if self == Self::RunPause {
            return if running {
                "Pause simulation"
            } else {
                "Run simulation"
            };
        }
        self.static_label()
    }

    /// Returns the icon shown next to the command.
    #[must_use]
    pub fn icon(self, running: bool) -> IconKind {
        if self == Self::RunPause {
            return if running {
                IconKind::Pause
            } else {
                IconKind::Run
            };
        }
        self.edit_icon()
            .or_else(|| self.view_icon())
            .unwrap_or_else(|| self.layout_icon())
    }

    /// Returns the keyboard shortcut hint, when one exists.
    #[must_use]
    pub fn shortcut(self) -> Option<&'static str> {
        self.edit_shortcut().or_else(|| self.view_shortcut())
    }

    fn edit_icon(self) -> Option<IconKind> {
        match self {
            Self::Save => Some(IconKind::Save),
            Self::Open => Some(IconKind::Open),
            Self::LoadHex => Some(IconKind::LoadHex),
            Self::Step => Some(IconKind::Step),
            Self::Reset => Some(IconKind::Reset),
            Self::Undo => Some(IconKind::Undo),
            Self::Redo => Some(IconKind::Redo),
            Self::Copy | Self::Duplicate => Some(IconKind::Copy),
            Self::Paste => Some(IconKind::Clipboard),
            Self::Delete => Some(IconKind::Trash),
            _ => None,
        }
    }

    fn view_icon(self) -> Option<IconKind> {
        match self {
            Self::ToggleGrid => Some(IconKind::Grid),
            Self::ToggleMinimap => Some(IconKind::Map),
            Self::ToggleLibrary => Some(IconKind::Folder),
            Self::ToggleInspector => Some(IconKind::Settings),
            Self::ToggleLegend => Some(IconKind::Palette),
            Self::ToggleWaveform => Some(IconKind::Chart),
            Self::ToggleUart => Some(IconKind::Binary),
            Self::ToggleLog => Some(IconKind::FileText),
            Self::ZoomReset => Some(IconKind::Fit),
            Self::ZoomUndo => Some(IconKind::Undo),
            Self::SelectTool => Some(IconKind::Pointer),
            Self::WireTool => Some(IconKind::Route),
            _ => None,
        }
    }

    fn layout_icon(self) -> IconKind {
        match self {
            Self::AlignLeft => IconKind::AlignLeft,
            Self::AlignRight => IconKind::AlignRight,
            Self::AlignTop => IconKind::AlignTop,
            Self::AlignBottom => IconKind::AlignBottom,
            Self::AlignCenterX | Self::AlignCenterY => IconKind::AlignCenter,
            Self::DistributeHorizontal => IconKind::DistributeHorizontal,
            Self::DistributeVertical => IconKind::DistributeVertical,
            _ => IconKind::Ruler,
        }
    }

    fn edit_shortcut(self) -> Option<&'static str> {
        match self {
            Self::Save => Some("Ctrl+S"),
            Self::Open => Some("Ctrl+O"),
            Self::Undo => Some("Ctrl+Z"),
            Self::Redo => Some("Ctrl+Shift+Z"),
            Self::Copy => Some("Ctrl+C"),
            Self::Paste => Some("Ctrl+V"),
            Self::Duplicate => Some("Ctrl+D"),
            Self::Delete => Some("Delete"),
            _ => None,
        }
    }

    fn view_shortcut(self) -> Option<&'static str> {
        match self {
            Self::ToggleLibrary => Some("L"),
            Self::ToggleInspector => Some("I"),
            Self::ToggleLegend => Some("K"),
            Self::SelectTool => Some("V"),
            Self::WireTool => Some("W"),
            _ => None,
        }
    }

    const fn static_label(self) -> &'static str {
        match self {
            Self::Copy => "Copy selection",
            Self::Paste => "Paste selection",
            Self::Duplicate => "Duplicate selection",
            _ => self.edit_label(),
        }
    }

    const fn edit_label(self) -> &'static str {
        match self {
            Self::Save => "Save project",
            Self::Open => "Open project",
            Self::LoadHex => "Load HEX firmware",
            Self::RunPause => "Run simulation",
            Self::Step => "Step instruction",
            Self::Reset => "Reset simulation",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            _ => self.view_label(),
        }
    }

    const fn view_label(self) -> &'static str {
        match self {
            Self::Delete => "Delete selection",
            Self::ToggleGrid => "Toggle grid",
            Self::ToggleMinimap => "Toggle minimap",
            Self::ToggleLibrary => "Toggle library",
            Self::ToggleInspector => "Toggle inspector",
            Self::ToggleLegend => "Toggle net legend",
            Self::ToggleWaveform => "Toggle waveform",
            Self::ToggleUart => "Toggle UART monitor",
            Self::ToggleLog => "Toggle log panel",
            Self::ZoomReset => "Reset zoom",
            Self::ZoomUndo => "Undo last zoom",
            Self::SelectTool => "Select tool",
            Self::WireTool => "Wire tool",
            _ => self.align_label(),
        }
    }

    const fn align_label(self) -> &'static str {
        match self {
            Self::AlignLeft => "Align left",
            Self::AlignRight => "Align right",
            Self::AlignTop => "Align top",
            Self::AlignBottom => "Align bottom",
            Self::AlignCenterX => "Align center horizontally",
            Self::AlignCenterY => "Align center vertically",
            Self::DistributeHorizontal => "Distribute horizontally",
            Self::DistributeVertical => "Distribute vertically",
            _ => "Unknown command",
        }
    }
}

/// Palette visibility and keyboard selection state.
#[derive(Default)]
pub struct State {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub focus: bool,
}

impl State {
    /// Toggles the palette and resets its query when opening.
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
            return;
        }
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.focus = true;
    }

    /// Closes the palette.
    pub fn close(&mut self) {
        self.open = false;
        self.focus = false;
    }

    /// Moves the selection with wrapping.
    pub fn move_selection(&mut self, count: usize, delta: isize) {
        self.selected = step_selection(self.selected, count, delta);
    }
}

/// Returns commands whose labels contain the query, ignoring case.
#[must_use]
pub fn matching(query: &str, running: bool) -> Vec<Command> {
    let needle = query.trim().to_lowercase();
    Command::ALL
        .into_iter()
        .filter(|command| {
            needle.is_empty() || command.label(running).to_lowercase().contains(&needle)
        })
        .collect()
}

/// Steps a selection index with wrapping.
#[must_use]
pub fn step_selection(index: usize, count: usize, delta: isize) -> usize {
    if count == 0 {
        return 0;
    }
    (index as isize + delta).rem_euclid(count as isize) as usize
}

#[cfg(test)]
mod tests {
    use super::{Command, State, matching, step_selection};
    use crate::icons::IconKind;

    #[test]
    fn filters_case_insensitive() {
        assert_eq!(matching("SAVE", false), vec![Command::Save]);
        assert!(matching("missing", false).is_empty());
        assert_eq!(matching("", false).len(), Command::ALL.len());
    }

    #[test]
    fn run_label_depends_on_state() {
        assert!(matching("pause", false).is_empty());
        assert_eq!(matching("pause", true), vec![Command::RunPause]);
        assert_eq!(matching("run", false), vec![Command::RunPause]);
    }

    #[test]
    fn commands_have_icons_and_shortcuts() {
        assert_eq!(Command::RunPause.icon(false), IconKind::Run);
        assert_eq!(Command::RunPause.icon(true), IconKind::Pause);
        assert_eq!(Command::Save.icon(false), IconKind::Save);
        assert_eq!(Command::Save.shortcut(), Some("Ctrl+S"));
        assert_eq!(Command::SelectTool.shortcut(), Some("V"));
        assert_eq!(Command::AlignLeft.shortcut(), None);
    }

    #[test]
    fn selection_wraps_both_ways() {
        assert_eq!(step_selection(0, 3, -1), 2);
        assert_eq!(step_selection(2, 3, 1), 0);
        assert_eq!(step_selection(0, 0, 1), 0);
    }

    #[test]
    fn toggle_resets_query_and_selection() {
        let mut state = State::default();
        state.query.push_str("save");
        state.selected = 4;
        state.toggle();
        assert!(state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.focus);
        state.toggle();
        assert!(!state.open);
    }
}
