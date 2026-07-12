use super::Action;
use crate::ui::{
    ScreenOverlay, ScreenOverlayResult, ScreenState, palette, utils::keybindmanager::KeybindManager,
};
use std::collections::HashMap;
use tui_logger::TuiWidgetState;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {}

pub struct State {
    app_actions: Vec<crate::Action>,
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    state: TuiWidgetState,
    log_file_path: String,
}

impl State {
    pub fn new() -> Self {
        Self {
            app_actions: vec![],
            log_file_path: crate::get_log_file_path()
                .expect("Get log file path")
                .to_string_lossy()
                .to_string(),
            state: TuiWidgetState::new(),
            overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("h", Action::Back),
                (":", Action::OpenCommandPalette),
            ])),
        }
    }

    pub fn scroll_state(&mut self) -> &mut TuiWidgetState {
        &mut self.state
    }

    pub fn log_file_path(&self) -> String {
        self.log_file_path.clone()
    }
}

impl ScreenState<Action, PaletteType, InputType> for State {
    fn update(&mut self) {}

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Back => {
                self.app_actions.push(crate::Action::Back);
            }
            Action::Quit => self.app_actions.push(crate::Action::Quit),

            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteType, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteType, InputType>) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => {}
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => self.apply_action(action),
            },
            ScreenOverlayResult::Input { value: _, typ: _ } => unreachable!(""),
        }
    }
}
