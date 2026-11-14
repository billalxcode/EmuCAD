use super::*;

/// Result of one canvas frame.
#[derive(Clone, Copy, Debug)]
pub struct CanvasState {
    /// Canvas rectangle in screen space.
    pub rect: egui::Rect,
    /// Pointer position in world grid space.
    pub pointer_world: Option<egui::Pos2>,
    /// True when the primary button was clicked on the canvas.
    pub clicked: bool,
    /// World position of the primary click.
    pub click_world: Option<egui::Pos2>,
    /// True when the secondary button was clicked on the canvas.
    pub secondary_clicked: bool,
    /// True when the primary button was double-clicked.
    pub double_clicked: bool,
    /// True when a primary drag started this frame.
    pub drag_started: bool,
    /// True while a primary drag is active.
    pub dragged: bool,
    /// True when the primary drag stopped this frame.
    pub drag_stopped: bool,
    /// True while the Space key is held.
    pub space_down: bool,
    /// True when Shift or Ctrl is held for additive selection.
    pub multi_select: bool,
    /// True when the canvas has keyboard focus.
    pub focused: bool,
    /// Position of a secondary click.
    pub context_click: Option<ContextClick>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            rect: egui::Rect::NOTHING,
            pointer_world: None,
            clicked: false,
            click_world: None,
            secondary_clicked: false,
            double_clicked: false,
            drag_started: false,
            dragged: false,
            drag_stopped: false,
            space_down: false,
            multi_select: false,
            focused: false,
            context_click: None,
        }
    }
}

/// Screen and world position of a context click.
#[derive(Clone, Copy, Debug)]
pub struct ContextClick {
    pub screen: egui::Pos2,
    pub world: egui::Pos2,
}

pub(super) fn canvas_state(
    ui: &egui::Ui,
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
    pointer_world: Option<egui::Pos2>,
) -> CanvasState {
    let space_down = ui.input(|input| input.key_down(egui::Key::Space));
    let mut state = CanvasState {
        rect,
        pointer_world,
        space_down,
        focused: response.has_focus(),
        ..CanvasState::default()
    };
    fill_interaction(&mut state, response, rect, camera);
    state
}

fn fill_interaction(
    state: &mut CanvasState,
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) {
    state.clicked = response.clicked();
    state.click_world = click_world(response, rect, camera);
    state.secondary_clicked = response.secondary_clicked();
    state.double_clicked = response.double_clicked();
    state.drag_started = response.drag_started_by(egui::PointerButton::Primary);
    state.dragged = response.dragged_by(egui::PointerButton::Primary);
    state.drag_stopped = response.drag_stopped_by(egui::PointerButton::Primary);
    state.multi_select = multi_select(response);
    state.context_click = context_click(response, rect, camera);
}

fn context_click(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<ContextClick> {
    let screen = interaction_screen(response)?;
    Some(ContextClick {
        screen,
        world: camera.screen_to_world(rect, screen),
    })
}

fn interaction_screen(response: &egui::Response) -> Option<egui::Pos2> {
    response
        .interact_pointer_pos()
        .or_else(|| response.hover_pos())
}

fn click_world(
    response: &egui::Response,
    rect: egui::Rect,
    camera: &GridCamera,
) -> Option<egui::Pos2> {
    response
        .interact_pointer_pos()
        .map(|position| camera.screen_to_world(rect, position))
}

fn multi_select(response: &egui::Response) -> bool {
    response
        .ctx
        .input(|input| input.modifiers.shift || input.modifiers.command)
}
