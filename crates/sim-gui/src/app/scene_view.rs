use super::*;

pub(crate) fn show_canvas(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid_visible: bool,
    initialized: &mut bool,
    scene: Scene<'_>,
) -> canvas::CanvasState {
    let draw = |painter: &egui::Painter,
                rect: egui::Rect,
                camera: &GridCamera,
                pointer: Option<egui::Pos2>| {
        draw_scene(painter, rect, camera, pointer, scene);
    };
    let show = |ui: &mut egui::Ui| canvas::show(ui, camera, grid_visible, initialized, draw);
    egui::CentralPanel::default().show(ui, show).inner
}

pub(crate) fn build_scene(refs: SceneRefs<'_>, state: SceneState) -> Scene<'_> {
    Scene {
        document: refs.document,
        library: refs.library,
        selection: refs.selection,
        hover_wire: state.hover_wire,
        placing: state.placing,
        rotation: state.rotation,
        mirror: state.mirror,
        wire_preview: refs.wire_preview,
        wire_class: state.wire_class,
        selection_box: state.selection_box,
        minimap: state.minimap,
        flow_phase: state.flow_phase,
    }
}

pub(crate) fn draw_scene(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &GridCamera,
    pointer: Option<egui::Pos2>,
    scene: Scene<'_>,
) {
    let hover = pointer.map(world_to_grid);
    scene::draw(painter, rect, camera, scene, hover);
}

pub(crate) fn show_component_info(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    definition: &ComponentDefinition,
) {
    ui.label(format!("{} {}", component.label, component.value));
    ui.label(format!("Type: {}", definition.name));
    ui.label(format!(
        "Pos: {}, {}",
        component.position.x, component.position.y
    ));
    ui.label(format!(
        "Rot: {}  Mirror: {}",
        component.rotation.label(),
        component.mirror.label()
    ));
    ui.separator();
    show_component_pins(ui, definition);
    show_component_simulation(ui);
}

pub(crate) fn show_component_value(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    definition: &ComponentDefinition,
    action: &mut Option<InspectorAction>,
) {
    let fields = crate::netlist::parameter_fields(definition);
    if fields.is_empty() {
        return;
    }
    ui.separator();
    ui.label("Parameters");
    for field in &fields {
        show_parameter(ui, component, field, action);
    }
}

fn show_parameter(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    field: &crate::netlist::ParameterField,
    action: &mut Option<InspectorAction>,
) {
    if field.is_boolean() {
        show_boolean_parameter(ui, component, field, action);
        return;
    }
    show_number_parameter(ui, component, field, action);
}

fn show_number_parameter(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    field: &crate::netlist::ParameterField,
    action: &mut Option<InspectorAction>,
) {
    let mut value = parameter_value(component, field);
    ui.horizontal(|ui| {
        ui.label(field.spec.name);
        let changed = ui.add(drag_value(&mut value, field)).changed();
        if !field.spec.unit.is_empty() {
            ui.label(field.spec.unit);
        }
        if changed {
            set_parameter(action, field, value);
        }
    });
}

fn show_boolean_parameter(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    field: &crate::netlist::ParameterField,
    action: &mut Option<InspectorAction>,
) {
    let mut checked = parameter_value(component, field) != 0.0;
    if ui.checkbox(&mut checked, field.spec.name).changed() {
        set_parameter(action, field, if checked { 1.0 } else { 0.0 });
    }
}

fn parameter_value(component: &ComponentInstance, field: &crate::netlist::ParameterField) -> f64 {
    component
        .parameters
        .get(field.spec.name)
        .copied()
        .unwrap_or(field.default)
}

fn drag_value<'a>(
    value: &'a mut f64,
    field: &crate::netlist::ParameterField,
) -> egui::DragValue<'a> {
    egui::DragValue::new(value)
        .range(field.spec.min..=field.spec.max)
        .speed(0.01)
}

fn set_parameter(
    action: &mut Option<InspectorAction>,
    field: &crate::netlist::ParameterField,
    value: f64,
) {
    *action = Some(InspectorAction::SetParameter {
        name: field.spec.name,
        value,
    });
}

pub(crate) fn show_component_simulation(ui: &mut egui::Ui) {
    ui.separator();
    ui.label("Simulation");
    ui.label("Global PB5/LED graph");
}

pub(crate) fn show_component_pins(ui: &mut egui::Ui, definition: &ComponentDefinition) {
    ui.label("Pins");
    for pin in definition.pins {
        ui.label(format!("{} ({})", pin.name, pin.number));
    }
}

pub(crate) fn show_component_actions(ui: &mut egui::Ui, action: &mut Option<InspectorAction>) {
    ui.horizontal(|ui| {
        if ui.button("Rotate").clicked() {
            *action = Some(InspectorAction::Rotate);
        }
        if ui.button("Mirror").clicked() {
            *action = Some(InspectorAction::Mirror);
        }
        if ui.button("Delete").clicked() {
            *action = Some(InspectorAction::Delete);
        }
    });
}
