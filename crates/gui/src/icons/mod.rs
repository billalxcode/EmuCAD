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
    label: &'static str,
}

/// Builds an icon button.
pub(crate) fn icon_button(icon: IconKind, label: &'static str) -> IconButton {
    IconButton { icon, label }
}

impl egui::Widget for IconButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.add(egui::Button::new(format!("      {}", self.label)));
        let color = ui.style().interact(&response).fg_stroke.color;
        let size = 14.0;
        let rect = egui::Rect::from_min_size(
            response.rect.left_center() + egui::vec2(6.0, -size * 0.5),
            egui::vec2(size, size),
        );
        draw_icon(ui.painter(), rect, self.icon, color);
        response
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
