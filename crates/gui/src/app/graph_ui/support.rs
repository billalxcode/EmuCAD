use super::*;
use crate::icons::IconKind;
pub(crate) use crate::icons::{IconButton, icon_button};

pub(crate) fn world_to_grid(position: egui::Pos2) -> GridPos {
    GridPos::new(position.x.round() as i32, position.y.round() as i32)
}

pub(crate) fn wire_color_message(has_color: bool) -> String {
    if has_color {
        return "Wire color updated".to_string();
    }
    "Wire color reset".to_string()
}

pub(crate) fn pb5_probe_spec() -> ProbeSpec {
    ProbeSpec {
        label: "PB5 voltage".to_string(),
        unit: "V",
        color: PB5_COLOR,
        source: ProbeSource::Pin {
            component: ComponentId(0),
            pin: "PB5",
        },
        sampling: Sampling::default(),
    }
}

pub(crate) fn led_probe_spec() -> ProbeSpec {
    ProbeSpec {
        label: "LED current".to_string(),
        unit: "A",
        color: LED_COLOR,
        source: ProbeSource::Branch {
            component: ComponentId(0),
            pin: "LED",
        },
        sampling: Sampling::default(),
    }
}

pub(crate) fn sample_combo(ui: &mut egui::Ui, current: u64) -> u64 {
    let mut every = current;
    egui::ComboBox::from_id_salt("sample_every")
        .selected_text(format!("N {every}"))
        .show_ui(ui, |ui| {
            for value in [10, 100, 1000] {
                ui.selectable_value(&mut every, value, format!("N {value}"));
            }
        });
    every
}

pub(crate) fn run_button_label(running: bool) -> IconButton {
    if running {
        icon_button(IconKind::Pause, "Pause")
    } else {
        icon_button(IconKind::Run, "Run")
    }
}

pub(crate) fn nudge_delta(ui: &egui::Ui) -> (i32, i32) {
    let step = if ui.input(|input| input.modifiers.shift) {
        5
    } else {
        1
    };
    let mut delta = (0, 0);
    ui.input(|input| {
        delta.0 = axis_delta(input, egui::Key::ArrowLeft, egui::Key::ArrowRight, step);
        delta.1 = axis_delta(input, egui::Key::ArrowUp, egui::Key::ArrowDown, step);
    });
    delta
}

pub(crate) fn axis_delta(
    input: &egui::InputState,
    negative: egui::Key,
    positive: egui::Key,
    step: i32,
) -> i32 {
    let mut delta = 0;
    if input.key_pressed(negative) {
        delta -= step;
    }
    if input.key_pressed(positive) {
        delta += step;
    }
    delta
}

pub(crate) fn flow_phase(enabled: bool, time: f64) -> Option<f32> {
    if !enabled {
        return None;
    }
    Some((time * 24.0).rem_euclid(1000.0) as f32)
}

pub(crate) fn grid_pos_f(position: GridPos) -> egui::Pos2 {
    egui::pos2(position.x as f32, position.y as f32)
}

pub(crate) fn content_bounds(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
) -> Option<(GridPos, GridPos)> {
    let mut min = None;
    let mut max = None;
    include_components(document, definitions, selection, &mut min, &mut max);
    include_wires(document, selection, &mut min, &mut max);
    Some((min?, max?))
}

pub(crate) fn include_components(
    document: &Document,
    definitions: &[ComponentDefinition],
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for instance in document.components() {
        if selection.is_some_and(|selection| !selection.contains_component(instance.id)) {
            continue;
        }
        let Some(bounds) = instance_bounds(definitions, instance) else {
            continue;
        };
        crate::model::extend_bounds(min, max, bounds.min);
        crate::model::extend_bounds(min, max, GridPos::new(bounds.max_x(), bounds.max_y()));
    }
}

pub(crate) fn instance_bounds(
    definitions: &[ComponentDefinition],
    instance: &ComponentInstance,
) -> Option<GridRect> {
    let definition = definitions
        .iter()
        .find(|definition| definition.id == instance.definition)?;
    Some(instance.bounds(definition))
}

pub(crate) fn include_wires(
    document: &Document,
    selection: Option<&Selection>,
    min: &mut Option<GridPos>,
    max: &mut Option<GridPos>,
) {
    for wire in document.wires() {
        if selection.is_some_and(|selection| !selection.contains_wire(wire.id)) {
            continue;
        }
        for point in &wire.points {
            crate::model::extend_bounds(min, max, *point);
        }
    }
}

pub(crate) fn move_segment(points: &mut [GridPos], index: usize, target: GridPos) -> bool {
    let Some(&end) = points.get(index + 1) else {
        return false;
    };
    let start = points[index];
    if start.y == end.y {
        points[index].y = target.y;
        points[index + 1].y = target.y;
    } else {
        points[index].x = target.x;
        points[index + 1].x = target.x;
    }
    true
}

pub(crate) fn l_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target {
        return Vec::new();
    }
    if last.x == target.x || last.y == target.y {
        return vec![target];
    }
    let corner = if horizontal_first {
        GridPos::new(target.x, last.y)
    } else {
        GridPos::new(last.x, target.y)
    };
    vec![corner, target]
}

pub(crate) fn z_route(last: GridPos, target: GridPos, horizontal_first: bool) -> Vec<GridPos> {
    if last == target || last.x == target.x || last.y == target.y {
        return l_route(last, target, horizontal_first);
    }
    let path = if horizontal_first {
        let middle = GridPos::new((last.x + target.x) / 2, last.y);
        vec![middle, GridPos::new(middle.x, target.y), target]
    } else {
        let middle = GridPos::new(last.x, (last.y + target.y) / 2);
        vec![middle, GridPos::new(target.x, middle.y), target]
    };
    without_repeats(last, path)
}

pub(crate) fn without_repeats(start: GridPos, path: Vec<GridPos>) -> Vec<GridPos> {
    let mut result = Vec::new();
    let mut previous = start;
    for point in path {
        if point == previous {
            continue;
        }
        result.push(point);
        previous = point;
    }
    result
}
