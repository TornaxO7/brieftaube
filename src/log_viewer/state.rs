use super::Action;
use crate::utils::ui::{
    ScreenOverlay, ScreenOverlayResult, ScreenState, keybindmanager::KeybindManager, palette,
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
    overlay: Option<ScreenOverlay<PaletteType, InputType>>,
    keybindings: KeybindManager<Action>,

    state: TuiWidgetState,
    log_file_path: String,
}

impl State {
    pub fn new() -> Self {
        Self {
            log_file_path: crate::get_log_file_path()
                .expect("Get log file path")
                .to_string_lossy()
                .to_string(),
            state: TuiWidgetState::new(),
            overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("h", Action::Back),
                ("<C-l>", Action::Back),
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

impl<'a> ScreenState<'a, Action, PaletteType, InputType, ()> for State {
    fn apply_user_action(&mut self, action: Action) -> Option<crate::Action> {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Back => return Some(crate::Action::Back),
            Action::Quit => return Some(crate::Action::Quit),

            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
        };

        None
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteType, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(
        &mut self,
        result: ScreenOverlayResult<PaletteType, InputType>,
    ) -> Option<crate::Action> {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => None,
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => self.apply_user_action(action),
            },
            ScreenOverlayResult::Input { value: _, typ: _ } => unreachable!(""),
        }
    }

    fn render_data(&mut self) {}

    fn update(&mut self) {}
}
