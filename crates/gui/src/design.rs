//! Adobe Spectrum inspired design tokens and egui style.

use eframe::egui;
use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

pub(crate) const SIZE_CAPTION: f32 = 11.0;
pub(crate) const SIZE_SMALL: f32 = 11.0;
pub(crate) const SIZE_BODY: f32 = 14.0;
pub(crate) const SIZE_BUTTON: f32 = 14.0;
pub(crate) const SIZE_HEADING: f32 = 18.0;
pub(crate) const SIZE_TITLE: f32 = 22.0;
pub(crate) const SIZE_MONO: f32 = 13.0;
pub(crate) const SIZE_PIN: f32 = 10.0;
pub(crate) const ROW_HEIGHT: f32 = 28.0;

const MEDIUM_NAME: &str = "medium";
const SEMIBOLD_NAME: &str = "semibold";
const BOLD_NAME: &str = "bold";

static MEDIUM_FAMILY: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(MEDIUM_NAME.into()));
static SEMIBOLD_FAMILY: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(SEMIBOLD_NAME.into()));
static BOLD_FAMILY: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(BOLD_NAME.into()));

/// Semantic color roles used by the UI and the canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Palette {
    pub background: egui::Color32,
    pub panel: egui::Color32,
    pub surface: egui::Color32,
    pub surface_raised: egui::Color32,
    pub surface_hover: egui::Color32,
    pub surface_active: egui::Color32,
    pub input: egui::Color32,
    pub faint: egui::Color32,
    pub border: egui::Color32,
    pub border_strong: egui::Color32,
    pub text: egui::Color32,
    pub text_strong: egui::Color32,
    pub text_muted: egui::Color32,
    pub text_disabled: egui::Color32,
    pub accent: egui::Color32,
    pub accent_hover: egui::Color32,
    pub accent_active: egui::Color32,
    pub focus: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub danger: egui::Color32,
    pub selection: egui::Color32,
    pub canvas: egui::Color32,
    pub grid_minor: egui::Color32,
    pub grid_major: egui::Color32,
    pub grid_axis: egui::Color32,
    pub ruler: egui::Color32,
    pub symbol: egui::Color32,
    pub pin: egui::Color32,
    pub junction: egui::Color32,
    pub graph_bg: egui::Color32,
    pub graph_grid: egui::Color32,
    pub shadow: egui::Color32,
}

pub(crate) const DARK: Palette = Palette {
    background: egui::Color32::from_rgb(27, 27, 27),
    panel: egui::Color32::from_rgb(27, 27, 27),
    surface: egui::Color32::from_rgb(44, 44, 44),
    surface_raised: egui::Color32::from_rgb(50, 50, 50),
    surface_hover: egui::Color32::from_rgb(57, 57, 57),
    surface_active: egui::Color32::from_rgb(68, 68, 68),
    input: egui::Color32::from_rgb(17, 17, 17),
    faint: egui::Color32::from_rgb(34, 34, 34),
    border: egui::Color32::from_rgb(68, 68, 68),
    border_strong: egui::Color32::from_rgb(109, 109, 109),
    text: egui::Color32::from_rgb(219, 219, 219),
    text_strong: egui::Color32::from_rgb(242, 242, 242),
    text_muted: egui::Color32::from_rgb(175, 175, 175),
    text_disabled: egui::Color32::from_rgb(109, 109, 109),
    accent: egui::Color32::from_rgb(86, 129, 255),
    accent_hover: egui::Color32::from_rgb(105, 149, 254),
    accent_active: egui::Color32::from_rgb(64, 105, 253),
    focus: egui::Color32::from_rgb(105, 149, 254),
    success: egui::Color32::from_rgb(14, 175, 98),
    warning: egui::Color32::from_rgb(224, 100, 0),
    danger: egui::Color32::from_rgb(255, 107, 107),
    selection: egui::Color32::from_rgba_premultiplied(22, 32, 64, 64),
    canvas: egui::Color32::from_rgb(27, 27, 27),
    grid_minor: egui::Color32::from_rgb(44, 44, 44),
    grid_major: egui::Color32::from_rgb(57, 57, 57),
    grid_axis: egui::Color32::from_rgb(109, 109, 109),
    ruler: egui::Color32::from_rgb(175, 175, 175),
    symbol: egui::Color32::from_rgb(219, 219, 219),
    pin: egui::Color32::from_rgb(242, 242, 242),
    junction: egui::Color32::from_rgb(242, 242, 242),
    graph_bg: egui::Color32::from_rgb(27, 27, 27),
    graph_grid: egui::Color32::from_rgb(57, 57, 57),
    shadow: egui::Color32::from_rgba_premultiplied(0, 0, 0, 96),
};

pub(crate) const LIGHT: Palette = Palette {
    background: egui::Color32::from_rgb(248, 248, 248),
    panel: egui::Color32::from_rgb(248, 248, 248),
    surface: egui::Color32::from_rgb(255, 255, 255),
    surface_raised: egui::Color32::from_rgb(243, 243, 243),
    surface_hover: egui::Color32::from_rgb(233, 233, 233),
    surface_active: egui::Color32::from_rgb(218, 218, 218),
    input: egui::Color32::from_rgb(255, 255, 255),
    faint: egui::Color32::from_rgb(243, 243, 243),
    border: egui::Color32::from_rgb(218, 218, 218),
    border_strong: egui::Color32::from_rgb(198, 198, 198),
    text: egui::Color32::from_rgb(41, 41, 41),
    text_strong: egui::Color32::from_rgb(19, 19, 19),
    text_muted: egui::Color32::from_rgb(113, 113, 113),
    text_disabled: egui::Color32::from_rgb(198, 198, 198),
    accent: egui::Color32::from_rgb(59, 99, 251),
    accent_hover: egui::Color32::from_rgb(75, 117, 255),
    accent_active: egui::Color32::from_rgb(29, 62, 207),
    focus: egui::Color32::from_rgb(59, 99, 251),
    success: egui::Color32::from_rgb(5, 131, 78),
    warning: egui::Color32::from_rgb(194, 78, 0),
    danger: egui::Color32::from_rgb(201, 37, 45),
    selection: egui::Color32::from_rgba_premultiplied(15, 25, 63, 64),
    canvas: egui::Color32::from_rgb(248, 248, 248),
    grid_minor: egui::Color32::from_rgb(233, 233, 233),
    grid_major: egui::Color32::from_rgb(218, 218, 218),
    grid_axis: egui::Color32::from_rgb(143, 143, 143),
    ruler: egui::Color32::from_rgb(113, 113, 113),
    symbol: egui::Color32::from_rgb(41, 41, 41),
    pin: egui::Color32::from_rgb(19, 19, 19),
    junction: egui::Color32::from_rgb(19, 19, 19),
    graph_bg: egui::Color32::from_rgb(255, 255, 255),
    graph_grid: egui::Color32::from_rgb(225, 225, 225),
    shadow: egui::Color32::from_rgba_premultiplied(0, 0, 0, 32),
};

pub(crate) const fn palette(dark: bool) -> Palette {
    if dark { DARK } else { LIGHT }
}

pub(crate) fn palette_for(context: &egui::Context) -> Palette {
    match context.theme() {
        egui::Theme::Dark => DARK,
        egui::Theme::Light => LIGHT,
    }
}

pub(crate) fn is_dark(context: &egui::Context) -> bool {
    matches!(context.theme(), egui::Theme::Dark)
}

pub(crate) fn medium(size: f32) -> egui::FontId {
    egui::FontId::new(size, MEDIUM_FAMILY.clone())
}

pub(crate) fn semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, SEMIBOLD_FAMILY.clone())
}

pub(crate) fn bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, BOLD_FAMILY.clone())
}

pub(crate) fn mono(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Monospace)
}

/// Installs the embedded Adobe open-source fonts.
pub(crate) fn install_fonts(context: &egui::Context) {
    context.set_fonts(font_definitions());
}

const FONT_DATA: [(&str, &[u8]); 5] = [
    (
        "source_sans_regular",
        include_bytes!("../assets/fonts/SourceSans3-Regular.ttf"),
    ),
    (
        "source_sans_medium",
        include_bytes!("../assets/fonts/SourceSans3-Medium.ttf"),
    ),
    (
        "source_sans_semibold",
        include_bytes!("../assets/fonts/SourceSans3-Semibold.ttf"),
    ),
    (
        "source_sans_bold",
        include_bytes!("../assets/fonts/SourceSans3-Bold.ttf"),
    ),
    (
        "source_code_pro_regular",
        include_bytes!("../assets/fonts/SourceCodePro-Regular.ttf"),
    ),
];

fn font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    add_font_data(&mut fonts);
    add_primary_fonts(&mut fonts);
    add_named(&mut fonts, MEDIUM_NAME, "source_sans_medium");
    add_named(&mut fonts, SEMIBOLD_NAME, "source_sans_semibold");
    add_named(&mut fonts, BOLD_NAME, "source_sans_bold");
    fonts
}

fn add_font_data(fonts: &mut egui::FontDefinitions) {
    for (name, bytes) in FONT_DATA {
        let font = egui::FontData::from_static(bytes);
        fonts.font_data.insert(name.to_owned(), Arc::new(font));
    }
}

fn add_primary_fonts(fonts: &mut egui::FontDefinitions) {
    add_primary(fonts, egui::FontFamily::Proportional, "source_sans_regular");
    add_primary(
        fonts,
        egui::FontFamily::Monospace,
        "source_code_pro_regular",
    );
}

fn add_primary(fonts: &mut egui::FontDefinitions, family: egui::FontFamily, name: &str) {
    let list = fonts.families.entry(family).or_default();
    list.retain(|font| font != name);
    list.insert(0, name.to_owned());
}

fn add_named(fonts: &mut egui::FontDefinitions, family: &str, font: &str) {
    let fallback = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mut list = vec![font.to_owned()];
    list.extend(fallback);
    fonts
        .families
        .insert(egui::FontFamily::Name(family.into()), list);
}

/// Applies the design system to both egui themes.
pub(crate) fn configure(context: &eframe::CreationContext<'_>) {
    configure_context(&context.egui_ctx);
}

pub(crate) fn configure_context(context: &egui::Context) {
    install_fonts(context);
    // The canvas owns Ctrl+wheel for horizontal pan.
    context.options_mut(|options| options.input_options.zoom_modifier = egui::Modifiers::NONE);
    context.style_mut_of(egui::Theme::Dark, |style| apply_style(style, DARK, true));
    context.style_mut_of(egui::Theme::Light, |style| apply_style(style, LIGHT, false));
    context.set_theme(egui::ThemePreference::Dark);
}

fn apply_style(style: &mut egui::Style, palette: Palette, dark: bool) {
    apply_typography(style);
    apply_spacing(style);
    style.visuals = visuals_for(palette, dark);
    style.animation_time = 0.15;
}

fn apply_typography(style: &mut egui::Style) {
    style.text_styles = text_styles();
    style.spacing.extra_text_line_spacing = 2.0;
}

fn text_styles() -> BTreeMap<egui::TextStyle, egui::FontId> {
    let mut styles = BTreeMap::new();
    styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(SIZE_SMALL, egui::FontFamily::Proportional),
    );
    styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(SIZE_BODY, egui::FontFamily::Proportional),
    );
    styles.insert(egui::TextStyle::Button, medium(SIZE_BUTTON));
    styles.insert(egui::TextStyle::Heading, semibold(SIZE_HEADING));
    styles.insert(egui::TextStyle::Monospace, mono(SIZE_MONO));
    styles.insert(egui::TextStyle::Name("title".into()), bold(SIZE_TITLE));
    styles.insert(
        egui::TextStyle::Name("caption".into()),
        egui::FontId::new(SIZE_CAPTION, egui::FontFamily::Proportional),
    );
    styles
}

fn apply_spacing(style: &mut egui::Style) {
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(8.0, 4.0);
    spacing.button_padding = egui::vec2(10.0, 4.0);
    spacing.window_margin = egui::Margin::same(12);
    spacing.menu_margin = egui::Margin::same(6);
    spacing.interact_size = egui::vec2(48.0, 24.0);
    spacing.slider_width = 120.0;
    spacing.combo_width = 140.0;
    spacing.text_edit_width = 220.0;
    spacing.icon_width = 16.0;
    spacing.icon_width_inner = 8.0;
    spacing.icon_spacing = 6.0;
    let mut scroll = egui::style::ScrollStyle::thin();
    scroll.bar_width = 8.0;
    scroll.handle_min_length = 24.0;
    spacing.scroll = scroll;
}

fn visuals_for(palette: Palette, dark: bool) -> egui::Visuals {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    apply_color_roles(&mut visuals, palette);
    apply_widgets(&mut visuals, palette);
    apply_shadows(&mut visuals, palette);
    apply_behavior(&mut visuals);
    visuals
}

fn apply_color_roles(visuals: &mut egui::Visuals, palette: Palette) {
    visuals.weak_text_color = Some(palette.text_muted);
    visuals.panel_fill = palette.panel;
    visuals.window_fill = palette.surface;
    visuals.window_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.window_corner_radius = egui::CornerRadius::same(6);
    visuals.menu_corner_radius = egui::CornerRadius::same(4);
    visuals.extreme_bg_color = palette.input;
    visuals.text_edit_bg_color = Some(palette.input);
    visuals.faint_bg_color = palette.faint;
    visuals.code_bg_color = palette.faint;
    visuals.selection.bg_fill = palette.selection;
    visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.hyperlink_color = palette.accent;
    visuals.warn_fg_color = palette.warning;
    visuals.error_fg_color = palette.danger;
}

fn apply_widgets(visuals: &mut egui::Visuals, palette: Palette) {
    visuals.widgets.noninteractive = widget(
        palette.surface,
        palette.surface,
        palette.border,
        palette.text,
        4,
    );
    visuals.widgets.inactive = widget(
        palette.surface,
        palette.surface_raised,
        palette.border,
        palette.text,
        4,
    );
    apply_active_widgets(visuals, palette);
}

fn apply_active_widgets(visuals: &mut egui::Visuals, palette: Palette) {
    apply_hover_widget(visuals, palette);
    apply_press_widgets(visuals, palette);
}

fn apply_hover_widget(visuals: &mut egui::Visuals, palette: Palette) {
    visuals.widgets.hovered = widget(
        palette.surface_raised,
        palette.surface_hover,
        palette.border_strong,
        palette.text_strong,
        4,
    );
}

fn apply_press_widgets(visuals: &mut egui::Visuals, palette: Palette) {
    visuals.widgets.active = widget(
        palette.surface_hover,
        palette.surface_active,
        palette.accent,
        palette.text_strong,
        4,
    );
    visuals.widgets.open = widget(
        palette.surface_raised,
        palette.surface_hover,
        palette.accent,
        palette.text_strong,
        4,
    );
}

fn widget(
    bg_fill: egui::Color32,
    weak_bg_fill: egui::Color32,
    border: egui::Color32,
    text: egui::Color32,
    radius: u8,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke: egui::Stroke::new(1.0, border),
        corner_radius: egui::CornerRadius::same(radius),
        fg_stroke: egui::Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

fn apply_behavior(visuals: &mut egui::Visuals) {
    visuals.striped = false;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);
    visuals.disabled_alpha = 0.45;
    visuals.slider_trailing_fill = true;
    visuals.window_highlight_topmost = false;
}

fn apply_shadows(visuals: &mut egui::Visuals, palette: Palette) {
    visuals.window_shadow = egui::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: palette.shadow,
    };
    visuals.popup_shadow = egui::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: palette.shadow,
    };
}

#[cfg(test)]
mod tests {
    use super::{DARK, LIGHT, font_definitions, palette};
    use eframe::egui;

    #[test]
    fn palette_switches_with_theme() {
        assert_eq!(palette(true), DARK);
        assert_eq!(palette(false), LIGHT);
    }

    #[test]
    fn text_contrast_is_readable() {
        for (name, palette) in [("dark", DARK), ("light", LIGHT)] {
            assert!(contrast(palette.text, palette.panel) >= 7.0, "{name} text");
            assert!(
                contrast(palette.text_muted, palette.panel) >= 4.5,
                "{name} muted"
            );
            for color in [
                palette.accent,
                palette.success,
                palette.warning,
                palette.danger,
            ] {
                assert!(contrast(color, palette.panel) >= 3.0, "{name} icon");
            }
        }
    }

    fn contrast(first: egui::Color32, second: egui::Color32) -> f64 {
        let first = luminance(first);
        let second = luminance(second);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    fn luminance(color: egui::Color32) -> f64 {
        let channel = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    #[test]
    fn configured_context_has_both_themes() {
        let context = egui::Context::default();
        super::configure_context(&context);
        let dark = context.style_of(egui::Theme::Dark);
        let light = context.style_of(egui::Theme::Light);
        assert_eq!(dark.visuals.panel_fill, DARK.panel);
        assert_eq!(light.visuals.panel_fill, LIGHT.panel);
        assert_eq!(dark.text_styles[&egui::TextStyle::Body].size, 14.0);
        assert_eq!(dark.text_styles[&egui::TextStyle::Heading].size, 18.0);
    }

    #[test]
    fn fonts_are_registered() {
        let fonts = font_definitions();
        for name in [
            "source_sans_regular",
            "source_sans_medium",
            "source_code_pro_regular",
        ] {
            assert!(fonts.font_data.contains_key(name));
        }
        for name in ["medium", "semibold"] {
            assert!(
                fonts
                    .families
                    .contains_key(&egui::FontFamily::Name(name.into()))
            );
        }
        assert_primary_font(&fonts);
    }

    fn assert_primary_font(fonts: &egui::FontDefinitions) {
        let list = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("proportional");
        assert_eq!(
            list.first().map(String::as_str),
            Some("source_sans_regular")
        );
    }
}
