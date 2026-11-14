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
