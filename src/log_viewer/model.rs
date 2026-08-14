use crate::utils::{
    keybindmanager::KeybindManager,
    layer::{LayerCore, LayerModel, LayerOverlay},
};

use super::Action;
use std::{collections::HashMap, str::FromStr};
use tui_logger::TuiWidgetState;

pub struct Model {
    keybindings: KeybindManager<Action>,

    state: TuiWidgetState,
    log_file_path: String,
}

impl Model {
    pub fn new() -> Self {
        Self {
            log_file_path: crate::get_log_file_path()
                .expect("Get log file path")
                .to_string_lossy()
                .to_string(),
            state: TuiWidgetState::new(),
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

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
        <Self as LayerModel<Action>>::handle_event(self, event, statusbar)
    }
}

impl LayerModel<Action> for Model {
    fn apply_action(&mut self, action: Action) -> Option<crate::Action> {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Back => Some(crate::Action::Back),
            Action::Quit => Some(crate::Action::Quit),

            Action::OpenCommandPalette => Some(crate::Action::OpenPalette {
                entries: super::action::palette_options(),
            }),
        }
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn handle_overlay<O>(&mut self, overlay: O) -> Option<crate::Action>
    where
        O: LayerOverlay,
    {
        let command_palette_msg = overlay.into_message().unwrap();
        let action = Action::from_str(&command_palette_msg.as_str()).unwrap();
        self.apply_action(action)
    }
}
