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

pub struct AttachmentsViewer {
    pub state: TableState,
    pub keybindings: KeybindManager<Action>,

    expected_overlay: Option<ExpectedOverlay>,
}

impl AttachmentsViewer {
    pub fn new() -> Self {
        Self {
            state: TableState::new(),
            expected_overlay: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::NavigateDown),
                ("k", Action::NavigateUp),
                (":", Action::OpenCommandPalette),
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
            Action::OpenCommandPalette => self.open_command_palette(),
            Action::Quit => self.quit(),
            Action::Back => self.back(),
            Action::NavigateDown => todo!(),
            Action::NavigateUp => todo!(),
            Action::NavigateToTop => todo!(),
            Action::NavigateToBottom => todo!(),
            Action::NavigateHalfPageDown => todo!(),
            Action::NavigateHalfPageUp => todo!(),
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

impl LayerModelDefaultHandleEvent<Action, super::Action> for AttachmentsViewer {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl MailViewerSubModel for AttachmentsViewer {}

impl AttachmentsViewer {
    fn open_command_palette(&mut self) -> Option<super::Action> {
        self.expected_overlay = Some(ExpectedOverlay::Action);
        Some(super::Action::OpenPalette {
            entries: action::palette_options(),
        })
    }
}
