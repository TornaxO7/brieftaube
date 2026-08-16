mod action;
mod widget;

use super::MailViewerSubModel;
use crate::utils::{
    keybindmanager::KeybindManager,
    layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent},
};
use ratatui::widgets::TableState;
use std::{collections::HashMap, str::FromStr};

pub use action::Action;
pub use widget::AttachmentsReader;

enum ExpectedOverlay {
    Action,
}

enum Navigate {
    Up(u16),
    Down(u16),
    HalfPageUp,
    HalfPageDown,
    Top,
    Bottom,
}

pub struct Model {
    pub state: TableState,
    pub keybindings: KeybindManager<Action>,

    expected_overlay: Option<ExpectedOverlay>,
    navigate: Option<Navigate>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            state: TableState::new(),
            expected_overlay: None,
            navigate: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::NavigateDown),
                ("k", Action::NavigateUp),
                (":", Action::OpenCommandPalette),
            ])),
        }
    }
}

impl LayerCore<super::Action> for Model {
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

impl LayerModel<Action, super::Action> for Model {
    fn apply_action(&mut self, action: Action) -> Option<super::Action> {
        match action {
            Action::OpenCommandPalette => self.open_command_palette(),
            Action::Quit => self.quit(),
            Action::Back => self.back(),
            Action::NavigateDown => self.navigate_down(),
            Action::NavigateUp => self.navigate_up(),
            Action::NavigateToTop => self.navigate_to_top(),
            Action::NavigateToBottom => self.navigate_to_bottom(),
            Action::NavigateHalfPageDown => self.navigate_half_page_down(),
            Action::NavigateHalfPageUp => self.navigate_half_page_up(),
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

impl LayerModelDefaultHandleEvent<Action, super::Action> for Model {
    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl MailViewerSubModel for Model {}

impl Model {
    fn open_command_palette(&mut self) -> Option<super::Action> {
        self.expected_overlay = Some(ExpectedOverlay::Action);
        Some(super::Action::OpenPalette {
            entries: action::palette_options(),
        })
    }

    fn navigate_down(&mut self) -> Option<super::Action> {
        let amount = self.keybindings.flush_int_prefix().unwrap_or(1);
        self.navigate = Some(Navigate::Down(amount as u16));
        None
    }

    fn navigate_up(&mut self) -> Option<super::Action> {
        let amount = self.keybindings.flush_int_prefix().unwrap_or(1);
        self.navigate = Some(Navigate::Up(amount as u16));
        None
    }

    fn navigate_to_top(&mut self) -> Option<super::Action> {
        self.navigate = Some(Navigate::Top);
        None
    }

    fn navigate_to_bottom(&mut self) -> Option<super::Action> {
        self.navigate = Some(Navigate::Bottom);
        None
    }

    fn navigate_half_page_down(&mut self) -> Option<super::Action> {
        self.navigate = Some(Navigate::HalfPageDown);
        None
    }

    fn navigate_half_page_up(&mut self) -> Option<super::Action> {
        self.navigate = Some(Navigate::HalfPageUp);
        None
    }
}
