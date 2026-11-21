use super::*;

pub(crate) fn show_canvas(
    ui: &mut egui::Ui,
    camera: &mut GridCamera,
    grid_visible: bool,
    grid_step: u32,
    initialized: &mut bool,
    scene: Scene<'_>,
) -> canvas::CanvasState {
    let draw = |painter: &egui::Painter,
                rect: egui::Rect,
                camera: &GridCamera,
                pointer: Option<egui::Pos2>| {
        draw_scene(painter, rect, camera, pointer, scene);
    };
    let show =
        |ui: &mut egui::Ui| canvas::show(ui, camera, grid_visible, grid_step, initialized, draw);
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
        snap: state.snap,
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
    let hover = scene
        .snap
        .map(|marker| marker.point)
        .or_else(|| pointer.map(world_to_grid));
    scene::draw(painter, rect, camera, scene, hover);
}

pub(crate) fn show_component_info(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    definition: &ComponentDefinition,
    action: &mut Option<InspectorAction>,
) {
    ui.label(format!("{} {}", component.label, component.value));
    ui.label(format!("Type: {}", definition.name));
    if let Some(fidelity) = crate::netlist::fidelity_label(definition) {
        ui.label(format!("Fidelity: {fidelity}"));
    }
    show_component_geometry(ui, component);
    ui.separator();
    show_component_pins(ui, definition);
    if definition.id == "atmega328p" {
        show_mcu_pin_modes(ui, component, definition, action);
    }
    show_component_simulation(ui);
}

fn show_component_geometry(ui: &mut egui::Ui, component: &ComponentInstance) {
    ui.label(format!(
        "Pos: {}, {}",
        component.position.x, component.position.y
    ));
    ui.label(format!(
        "Rot: {}  Mirror: {}",
        component.rotation.label(),
        component.mirror.label()
    ));
}

fn show_mcu_pin_modes(
    ui: &mut egui::Ui,
    component: &ComponentInstance,
    definition: &ComponentDefinition,
    action: &mut Option<InspectorAction>,
) {
    ui.separator();
    ui.label("MCU pin modes");
    let package = avr::AvrPackageMap::atmega328p_tqfp32();
    for pin in definition.pins {
        let Some((number, current)) = mcu_pin_state(pin, component, &package) else {
            continue;
        };
        show_mcu_pin_row(ui, pin, number, current, action);
    }
}

fn show_mcu_pin_row(
    ui: &mut egui::Ui,
    pin: &crate::model::PinDefinition,
    number: u8,
    current: McuPinMode,
    action: &mut Option<InspectorAction>,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{} {}", pin.number, pin.name));
        let selected = mcu_pin_mode_combo(ui, number, current);
        if selected != current {
            *action = Some(InspectorAction::SetPinMode {
                pin: number,
                mode: selected,
            });
        }
    });
}

fn mcu_pin_state(
    pin: &crate::model::PinDefinition,
    component: &ComponentInstance,
    package: &avr::AvrPackageMap,
) -> Option<(u8, McuPinMode)> {
    let number = pin.number.parse().ok()?;
    if !matches!(package.get(number), Some(avr::AvrSignal::Io { .. })) {
        return None;
    }
    let current = component
        .pin_modes
        .get(&number)
        .copied()
        .unwrap_or_default();
    Some((number, current))
}

fn mcu_pin_mode_combo(ui: &mut egui::Ui, number: u8, current: McuPinMode) -> McuPinMode {
    let mut selected = current;
    egui::ComboBox::from_id_salt(("mcu_pin_mode", number))
        .selected_text(selected.label())
        .show_ui(ui, |ui| {
            for mode in McuPinMode::ALL {
                ui.selectable_value(&mut selected, mode, mode.label());
            }
        });
    selected
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
    let roles = crate::netlist::pin_roles(definition);
    for (index, pin) in definition.pins.iter().enumerate() {
        let role = roles.get(index).map_or("", |role| role.label());
        ui.label(format!("{} ({}) {}", pin.name, pin.number, role));
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
