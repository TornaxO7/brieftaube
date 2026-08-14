mod action;

use crate::utils::{
    keybindmanager::KeybindManager,
    layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent},
};
use ratatui::widgets::TableState;
use std::collections::HashMap;

pub use action::Action;

pub struct MetadataViewer {
    pub state: TableState,
    pub keybindings: KeybindManager<Action>,
}

impl MetadataViewer {
    pub fn new() -> Self {
        Self {
            state: TableState::new(),
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
            ])),
        }
    }
}

impl LayerCore<super::Action> for MetadataViewer {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<super::Action> {
        <Self as LayerModelDefaultHandleEvent<Action, super::Action>>::handle_event(
            self, event, statusbar,
        )
    }
}

impl LayerModel<Action, super::Action> for MetadataViewer {
    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::OpenCommandPalette => todo!(),
            Action::Quit => todo!(),
            Action::Back => todo!(),
            Action::ScrollDown => todo!(),
            Action::ScrollUp => todo!(),
            Action::ScrollLeft => todo!(),
            Action::ScrollRight => todo!(),
            Action::ScrollToTop => todo!(),
            Action::ScrollToBottom => todo!(),
            Action::ScrollHalfPageDown => todo!(),
            Action::ScrollHalfPageUp => todo!(),
            Action::ScrollHalfPageRight => todo!(),
            Action::ScrollHalfPageLeft => todo!(),
            Action::OpenMetadataTab => todo!(),
            Action::OpenTextTab => todo!(),
            Action::OpenMarkdownTab => todo!(),
            Action::OpenAttachmentsTab => todo!(),
            Action::OpenNextTab => todo!(),
            Action::OpenPreviousTab => todo!(),
            Action::OpenLogs => todo!(),
        }
    }

    fn handle_overlay<O>(&mut self, overlay: O) -> Option<super::Action>
    where
        O: crate::utils::layer::LayerOverlay,
    {
        todo!()
    }
}

impl LayerModelDefaultHandleEvent<Action, super::Action> for MetadataViewer {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}
