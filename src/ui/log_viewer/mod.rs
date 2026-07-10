mod action;
mod widget;

use crate::ui::{ScreenPalette, ScreenState, keybindmanager::KeybindManager, palette};
pub use action::Action;
pub use widget::LogViewer;

use std::collections::HashMap;
use tui_logger::TuiWidgetState;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<Action>,

    state: TuiWidgetState,
    log_file_path: String,
    callback: Box<super::Action>,
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
            palette: None,
            callback: Box::new(super::Action::Quit),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
            ])),
        }
    }

    pub fn set_callback(&mut self, callback: Box<super::Action>) {
        self.callback = callback;
    }
}

impl ScreenState<Action, PaletteType> for State {
    fn update(&mut self) {}

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Back => {
                todo!()
                // return Some(*self.callback.clone()),
            }
            Action::Quit => self.app_actions.push(crate::Action::Quit),

            Action::CloseCommandPalette => self.palette = None,
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(action::palette_options()));
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl ScreenPalette<PaletteType> for State {
    fn palette(&mut self) -> Option<&mut palette::State<PaletteType>> {
        self.palette.as_mut()
    }

    fn handle_palette_result(&mut self, result: palette::HandleEventResult<PaletteType>) {
        match result {
            palette::HandleEventResult::Cancel => {}
            palette::HandleEventResult::Selected(value) => match value {
                PaletteType::Action(action) => self.apply_action(action),
            },
        }
    }
}
