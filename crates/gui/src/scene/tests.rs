use super::{
    minimap_map, net_color, pin_label_text, pin_tooltip, placement_position, preview_rect,
    segment_visible, show_component_labels, show_wire_label, wire_midpoint,
};
use crate::library;
use crate::model::{Document, GridPos, Mirror, NetClass, Rotation, Wire, WireId};
use crate::selection::Selection;

#[test]
fn component_labels_use_lod_except_selection() {
    assert!(show_component_labels(true, 0.4));
    assert!(show_component_labels(false, 0.7));
    assert!(!show_component_labels(false, 0.69));
}

#[test]
fn pin_tooltip_reports_name_and_side() {
    let pin = crate::model::PinDefinition {
        name: "A",
        number: "1",
        offset: GridPos::new(0, 1),
        direction: crate::model::PinDirection::Left,
    };
    assert_eq!(pin_tooltip(&pin, GridPos::new(-1, 0)), "A (1) left");
    assert_eq!(pin_label_text(&pin), "A/1");
}

#[test]
fn preview_rect_fits_library_area() {
    let definition = library::find("resistor").expect("resistor");
    let area = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(100.0, 60.0),
    );
    let body = preview_rect(area, definition);
    assert!(area.contains_rect(body));
    assert!((body.width() / body.height() - 2.0).abs() < 0.01);
}

#[test]
fn minimap_centers_world_bounds() {
    let area = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(160.0, 110.0),
    );
    let map = minimap_map(area, GridPos::new(0, 0), GridPos::new(10, 10)).expect("map");
    let center = map.point(GridPos::new(5, 5));
    assert!((center - area.center()).length() < 1e-4);
    assert!(map.scale > 0.0);
}

#[test]
fn scene_renders_in_egui_context() {
    let output = sample_render_with(None);
    assert!(!output.shapes.is_empty());
}

#[test]
fn flow_overlay_renders_in_egui_context() {
    let output = sample_render_with(Some(12.0));
    assert!(!output.shapes.is_empty());
}

fn sample_render_with(flow_phase: Option<f32>) -> eframe::egui::FullOutput {
    let library = library::builtin();
    let resistor = crate::library::find("resistor").expect("resistor");
    let mut document = Document::new();
    document.add(resistor, GridPos::new(0, 0), Rotation::Deg0, Mirror::None);
    let selection = Selection::default();
    let camera = crate::canvas::GridCamera::default();
    let mut scene = sample_scene(&document, library, &selection);
    scene.flow_phase = flow_phase;
    let input = test_input();
    let mut output = eframe::egui::Context::default().run_ui(input, |ui| {
        render_into(ui, scene, &camera);
    });
    output.textures_delta.clear();
    output
}

fn test_input() -> eframe::egui::RawInput {
    eframe::egui::RawInput {
        screen_rect: Some(eframe::egui::Rect::from_min_size(
            eframe::egui::Pos2::ZERO,
            eframe::egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}

fn render_into(
    ui: &mut eframe::egui::Ui,
    scene: super::Scene<'_>,
    camera: &crate::canvas::GridCamera,
) {
    let (response, painter) = ui.allocate_painter(
        eframe::egui::vec2(400.0, 300.0),
        eframe::egui::Sense::hover(),
    );
    super::draw(&painter, response.rect, camera, scene, None);
}

fn sample_scene<'a>(
    document: &'a Document,
    library: &'a [crate::model::ComponentDefinition],
    selection: &'a Selection,
) -> super::Scene<'a> {
    let mut scene = super::Scene::new((document, library, selection));
    scene.minimap = true;
    scene
}

#[test]
fn segment_visibility_rejects_offscreen_wires() {
    let camera = crate::canvas::GridCamera::default();
    let rect = eframe::egui::Rect::from_min_size(
        eframe::egui::pos2(0.0, 0.0),
        eframe::egui::vec2(200.0, 200.0),
    );
    assert!(segment_visible(
        rect,
        &camera,
        GridPos::new(0, 0),
        GridPos::new(2, 0)
    ));
    assert!(!segment_visible(
        rect,
        &camera,
        GridPos::new(100, 0),
        GridPos::new(102, 0)
    ));
}

#[test]
fn wire_label_visible_on_hover_selection_or_zoom() {
    assert!(show_wire_label(true, false, 0.5));
    assert!(show_wire_label(false, true, 0.5));
    assert!(show_wire_label(false, false, 1.5));
    assert!(!show_wire_label(false, false, 1.0));
}

#[test]
fn wire_midpoint_uses_middle_segment() {
    let wire = Wire {
        id: WireId(1),
        points: vec![GridPos::new(0, 0), GridPos::new(4, 0), GridPos::new(4, 4)],
        class: NetClass::Digital,
        color_override: None,
        label: None,
    };
    assert_eq!(wire_midpoint(&wire), Some(eframe::egui::pos2(4.0, 2.0)));
    let short = Wire {
        id: WireId(2),
        points: vec![GridPos::new(0, 0)],
        class: NetClass::Digital,
        color_override: None,
        label: None,
    };
    assert_eq!(wire_midpoint(&short), None);
}

#[test]
fn net_colors_are_distinct() {
    let colors: Vec<_> = NetClass::ALL
        .into_iter()
        .map(|class| net_color(class, true))
        .collect();
    for (index, color) in colors.iter().enumerate() {
        for other in &colors[index + 1..] {
            assert_ne!(color, other);
        }
    }
}

#[test]
fn wire_color_uses_override() {
    let wire = Wire {
        id: WireId(1),
        points: vec![GridPos::new(0, 0), GridPos::new(2, 0)],
        class: NetClass::Digital,
        color_override: Some([1, 2, 3]),
        label: Some("LED_A".to_string()),
    };
    assert_eq!(
        super::wire_color(&wire, true),
        eframe::egui::Color32::from_rgb(1, 2, 3)
    );
    assert_eq!(super::wire_label_text(&wire), "LED_A");
}

#[test]
fn net_colors_meet_wcag_aa_on_canvas() {
    for (dark, background) in [
        (true, crate::design::DARK.canvas),
        (false, crate::design::LIGHT.canvas),
    ] {
        let background = luminance(background);
        for class in NetClass::ALL {
            let foreground = luminance(net_color(class, dark));
            let ratio = (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
            assert!(ratio >= 4.5, "{} contrast {ratio:.2}", class.label());
        }
    }
}

fn luminance(color: eframe::egui::Color32) -> f64 {
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

fn channel(value: u8) -> f64 {
    let value = f64::from(value) / 255.0;
    if value <= 0.04045 {
        return value / 12.92;
    }
    ((value + 0.055) / 1.055).powf(2.4)
}

#[test]
fn placement_centers_component() {
    let definition = library::find("resistor").expect("resistor");
    let position = placement_position(definition, GridPos::new(10, 10), Rotation::Deg0);
    assert_eq!(position, GridPos::new(8, 9));
}

#[test]
fn placement_uses_rotated_size() {
    let definition = library::find("resistor").expect("resistor");
    let position = placement_position(definition, GridPos::new(10, 10), Rotation::Deg90);
    assert_eq!(position, GridPos::new(9, 8));
}
