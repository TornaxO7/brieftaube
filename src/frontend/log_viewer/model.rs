use crate::utils::{
    keybindmanager::KeybindManager,
    layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent, LayerOverlay},
};

use super::Action;
use arboard::Clipboard;
use std::{collections::HashMap, str::FromStr};
use tracing::error;
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
        <Self as LayerModelDefaultHandleEvent<Action>>::handle_event(self, event, statusbar)
    }
}

impl LayerModel<Action> for Model {
    fn apply_action(&mut self, action: Action) -> Option<crate::Action> {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Back => self.back(),
            Action::Quit => self.quit(),

            Action::CopyPathToClipboard => self.copy_path_to_clipboard(),
            Action::OpenCommandPalette => self.open_command_palette(),
        }
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

impl LayerModelDefaultHandleEvent<Action> for Model {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl Model {
    fn back(&self) -> Option<crate::Action> {
        Some(crate::Action::Back)
    }

    fn quit(&self) -> Option<crate::Action> {
        Some(crate::Action::Quit)
    }

    fn copy_path_to_clipboard(&self) -> Option<crate::Action> {
        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(err) => {
                error!("Can't open clipboard:\n{err}");
                return None;
            }
        };

        let log_file_path = match crate::get_log_file_path() {
            Ok(path) => path,
            Err(err) => {
                error!("Couldn't get log file path: {err}");
                return None;
            }
        };

        if let Err(err) = clipboard.set_text(log_file_path.to_string_lossy()) {
            error!("Couldn't copy path to log file into clipboard: {err}");
            return None;
        }

        None
    }

    fn open_command_palette(&self) -> Option<crate::Action> {
        Some(crate::Action::OpenPalette {
            entries: super::action::palette_options(),
        })
    }
}
