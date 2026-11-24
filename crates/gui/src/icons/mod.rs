//! Tabler icon assets and painting.

use crate::symbols::{SvgSymbol, draw_svg, parse_svg};
use eframe::egui;
use std::collections::BTreeMap;
use std::sync::OnceLock;

macro_rules! icon_assets {
    ($($variant:ident => $file:literal),* $(,)?) => {
        /// One vendored Tabler icon.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub(crate) enum IconKind {
            $($variant),*
        }

        impl IconKind {
            /// All vendored icons.
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),*];

            fn text(self) -> &'static str {
                match self {
                    $(Self::$variant => include_str!(concat!("../../assets/icons/", $file)),)*
                }
            }
        }
    };
}

icon_assets! {
    Undo => "arrow-back-up.svg",
    Redo => "arrow-forward-up.svg",
    Open => "folder-open.svg",
    Save => "device-floppy.svg",
    Png => "photo.svg",
    LoadHex => "file-import.svg",
    Netlist => "list-check.svg",
    Dc => "activity.svg",
    Transient => "wave-sine.svg",
    Run => "player-play.svg",
    Pause => "player-pause.svg",
    Step => "player-skip-forward.svg",
    Reset => "refresh.svg",
    ZoomIn => "zoom-in.svg",
    ZoomOut => "zoom-out.svg",
    Fit => "focus-2.svg",
    Grid => "grid-dots.svg",
    Map => "map.svg",
    Search => "search.svg",
    Settings => "settings.svg",
    Help => "help-circle.svg",
    Alert => "alert-triangle.svg",
    Check => "circle-check.svg",
    Close => "x.svg",
    Cpu => "cpu.svg",
    Resistor => "circuit-resistor.svg",
    Capacitor => "circuit-capacitor.svg",
    Diode => "circuit-diode.svg",
    Inductor => "circuit-inductor.svg",
    Database => "database.svg",
    Chart => "chart-line.svg",
    Bug => "bug.svg",
    Crosshair => "crosshair.svg",
    Math => "math-function.svg",
    Ruler => "ruler-measure.svg",
    Binary => "binary.svg",
    FileText => "file-text.svg",
    Folder => "folder.svg",
    Package => "package.svg",
    Trash => "trash.svg",
    Copy => "copy.svg",
    Cut => "cut.svg",
    Clipboard => "clipboard.svg",
    Sun => "sun.svg",
    Moon => "moon.svg",
    Pointer => "pointer.svg",
    Route => "route.svg",
    Palette => "palette.svg",
    Export => "file-export.svg",
    AlignLeft => "layout-align-left.svg",
    AlignRight => "layout-align-right.svg",
    AlignTop => "layout-align-top.svg",
    AlignBottom => "layout-align-bottom.svg",
    AlignCenter => "layout-align-center.svg",
    DistributeHorizontal => "layout-distribute-horizontal.svg",
    DistributeVertical => "layout-distribute-vertical.svg",
    Battery => "circuit-battery.svg",
    Transistor => "circuit-changeover.svg",
    Shield => "shield.svg",
    SwitchClosed => "circuit-switch-closed.svg",
    Bolt => "bolt.svg",
    Temperature => "temperature.svg",
    Engine => "engine.svg",
    Antenna => "antenna.svg",
    History => "history.svg",
    Filter => "filter.svg",
}

static ICONS: OnceLock<BTreeMap<IconKind, SvgSymbol>> = OnceLock::new();

fn icons() -> &'static BTreeMap<IconKind, SvgSymbol> {
    ICONS.get_or_init(|| {
        IconKind::ALL
            .iter()
            .filter_map(|&kind| {
                let symbol = parse_svg(kind.text()).ok()?;
                Some((kind, symbol))
            })
            .collect()
    })
}

/// Paints one icon into a rectangle.
pub(crate) fn draw_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    kind: IconKind,
    color: egui::Color32,
) {
    let Some(symbol) = icons().get(&kind) else {
        return;
    };
    draw_svg(painter, rect, symbol, color);
}

/// A button with one Tabler icon and a text label.
pub(crate) struct IconButton {
    icon: IconKind,
    label: String,
    selected: bool,
    tooltip: Option<String>,
    shortcut: Option<&'static str>,
}

/// Builds an icon button.
pub(crate) fn icon_button(icon: IconKind, label: impl Into<String>) -> IconButton {
    IconButton {
        icon,
        label: label.into(),
        selected: false,
        tooltip: None,
        shortcut: None,
    }
}

impl IconButton {
    /// Marks the button as selected.
    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Overrides the default tooltip.
    pub(crate) fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Adds a shortcut hint to the tooltip.
    pub(crate) fn shortcut(mut self, shortcut: &'static str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    fn hint(&self) -> String {
        if let Some(tooltip) = &self.tooltip {
            return tooltip.clone();
        }
        match self.shortcut {
            Some(shortcut) => format!("{} ({shortcut})", self.label),
            None => self.label.clone(),
        }
    }
}

impl egui::Widget for IconButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let button = egui::Button::new(format!("      {}", self.label)).selected(self.selected);
        let response = ui.add(button);
        let color = ui.style().interact(&response).fg_stroke.color;
        let size = 14.0;
        let rect = egui::Rect::from_min_size(
            response.rect.left_center() + egui::vec2(6.0, -size * 0.5),
            egui::vec2(size, size),
        );
        draw_icon(ui.painter(), rect, self.icon, color);
        let hint = self.hint();
        response.on_hover_text(hint)
    }
}

#[cfg(test)]
mod tests {
    use super::{IconKind, draw_icon, icons};

    #[test]
    fn all_icons_parse() {
        assert_eq!(icons().len(), IconKind::ALL.len());
    }

    #[test]
    fn draw_icon_paints_known_asset() {
        let context = eframe::egui::Context::default();
        let mut output = context.run_ui(Default::default(), |context| {
            let painter = context.layer_painter(eframe::egui::LayerId::background());
            draw_icon(
                &painter,
                eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(16.0, 16.0),
                ),
                IconKind::Cpu,
                eframe::egui::Color32::WHITE,
            );
        });
        output.textures_delta.clear();
    }
}
