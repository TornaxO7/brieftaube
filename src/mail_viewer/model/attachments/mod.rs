mod action;

use crate::utils::{
    keybindmanager::KeybindManager,
    layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent},
};
use ratatui::widgets::TableState;
use std::collections::HashMap;

pub use action::Action;

pub struct AttachmentsViewer {
    pub state: TableState,
    pub keybindings: KeybindManager<Action>,
}

impl AttachmentsViewer {
    pub fn new() -> Self {
        Self {
            state: TableState::new(),
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::NavigateDown),
                ("k", Action::NavigateUp),
            ])),
        }
    }
}

impl LayerCore<super::Action> for AttachmentsViewer {
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

impl LayerModel<Action, super::Action> for AttachmentsViewer {
    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::OpenCommandPalette => todo!(),
            Action::Quit => todo!(),
            Action::Back => todo!(),
            Action::NavigateDown => todo!(),
            Action::NavigateUp => todo!(),
            Action::NavigateToTop => todo!(),
            Action::NavigateToBottom => todo!(),
            Action::NavigateHalfPageDown => todo!(),
            Action::NavigateHalfPageUp => todo!(),
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

impl LayerModelDefaultHandleEvent<Action, super::Action> for AttachmentsViewer {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}
