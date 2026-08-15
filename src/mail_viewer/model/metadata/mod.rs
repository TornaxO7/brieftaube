mod action;

use crate::{
    mail_viewer::model::MailViewerSubModel,
    utils::{
        keybindmanager::KeybindManager,
        layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent},
    },
};
use ratatui::widgets::TableState;
use std::{collections::HashMap, str::FromStr};

pub use action::Action;

enum ExpectedOverlay {
    Action,
}

pub struct MetadataViewer {
    pub state: TableState,
    pub keybindings: KeybindManager<Action>,

    expected_overlay: Option<ExpectedOverlay>,
}

impl MetadataViewer {
    pub fn new() -> Self {
        Self {
            state: TableState::new(),
            expected_overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                (":", Action::OpenCommandPalette),
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
            Action::OpenCommandPalette => self.open_command_palette(),
            Action::Quit => self.quit(),
            Action::Back => self.back(),
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
            Action::OpenMetadataTab => self.open_metadata_tab(),
            Action::OpenTextTab => self.open_text_tab(),
            Action::OpenMarkdownTab => self.open_markdown_tab(),
            Action::OpenAttachmentsTab => self.open_attachments_tab(),
            Action::OpenNextTab => self.open_next_tab(),
            Action::OpenPreviousTab => self.open_previous_tab(),
            Action::OpenLogs => self.open_logs(),
        }
    }

    fn handle_overlay<O>(&mut self, overlay: O) -> Option<super::Action>
    where
        O: crate::utils::layer::LayerOverlay,
    {
        let expected_overlay = self.expected_overlay.take()?;
        let msg = overlay.into_message()?;

        match expected_overlay {
            ExpectedOverlay::Action => {
                let action = Action::from_str(msg.as_str()).unwrap();
                self.apply_action(action)
            }
        }
    }
}

impl LayerModelDefaultHandleEvent<Action, super::Action> for MetadataViewer {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl MailViewerSubModel for MetadataViewer {}

impl MetadataViewer {
    fn open_command_palette(&mut self) -> Option<super::Action> {
        self.expected_overlay = Some(ExpectedOverlay::Action);
        Some(super::Action::OpenPalette {
            entries: action::palette_options(),
        })
    }
}
