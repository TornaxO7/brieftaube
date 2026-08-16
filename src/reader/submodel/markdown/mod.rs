mod action;
mod widget;

use super::MailViewerSubModel;
use crate::{
    reader::submodel::{MailViewerPager, ScrollAction},
    utils::{
        keybindmanager::KeybindManager,
        layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent, LayerOverlay},
    },
};
use ratatui::widgets::ScrollbarState;
use std::{collections::HashMap, str::FromStr};

pub use action::Action;
pub use widget::MarkdownReader;

enum ExpectedOverlay {
    Action,
}

pub struct Model {
    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,

    pub keybindings: KeybindManager<Action>,

    expected_overlay: Option<ExpectedOverlay>,
    scroll_action: Option<ScrollAction>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            vertical: ScrollbarState::default(),
            horizontal: ScrollbarState::default(),
            expected_overlay: None,
            scroll_action: None,

            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("h", Action::Back),
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("zh", Action::ScrollLeft),
                ("zl", Action::ScrollRight),
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
            Action::ScrollDown => self.scroll_down(),
            Action::ScrollUp => self.scroll_up(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ScrollRight => self.scroll_right(),
            Action::ScrollToTop => self.scroll_to_top(),
            Action::ScrollToBottom => self.scroll_to_bottom(),
            Action::ScrollHalfPageDown => self.scroll_half_page_down(),
            Action::ScrollHalfPageUp => self.scroll_half_page_up(),
            Action::ScrollHalfPageRight => self.scroll_half_page_right(),
            Action::ScrollHalfPageLeft => self.scroll_half_page_left(),
            Action::OpenMetadataTab => self.open_metadata_tab(),
            Action::OpenTextTab => self.open_text_tab(),
            Action::OpenMarkdownTab => self.open_markdown_tab(),
            Action::OpenAttachmentsTab => self.open_attachments_tab(),
            Action::OpenNextTab => self.open_next_tab(),
            Action::OpenPreviousTab => self.open_previous_tab(),
            Action::OpenLogs => self.open_logs(),
            Action::OpenMailInBrowser => self.open_mail_in_browser(),
        }
    }

    fn handle_overlay<O>(&mut self, overlay: O) -> Option<super::Action>
    where
        O: LayerOverlay,
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

impl MailViewerPager<Action> for Model {
    fn set_scroll_action(&mut self, scroll: ScrollAction) {
        self.scroll_action = Some(scroll);
    }
}

impl Model {
    fn open_command_palette(&mut self) -> Option<super::Action> {
        self.expected_overlay = Some(ExpectedOverlay::Action);
        Some(super::Action::OpenPalette {
            entries: action::palette_options(),
        })
    }

    fn open_mail_in_browser(&self) -> Option<super::Action> {
        todo!()
    }
}
