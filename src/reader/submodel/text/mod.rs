mod action;
mod widget;

use super::SubModel;
use crate::{
    backend::{Backend, MailBodyType, MailId},
    reader::submodel::{MailContentReader, ScrollAction},
    task_manager::TaskManager,
    utils::{
        keybindmanager::KeybindManager,
        layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent, LayerOverlay},
    },
};
use ratatui::widgets::ScrollbarState;
use std::{collections::HashMap, rc::Rc, str::FromStr, sync::Arc};
use throbber_widgets_tui::ThrobberState;
use tracing::error;

pub use action::Action;
pub use widget::TextReader;

enum ExpectedOverlay {
    Action,
}

pub struct Model {
    id: MailId,
    backend: Arc<Backend>,
    task_manager: Rc<TaskManager>,

    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
    pub throbber: ThrobberState,

    pub keybindings: KeybindManager<Action>,

    expected_overlay: Option<ExpectedOverlay>,
    scroll_action: Option<ScrollAction>,
}

impl Model {
    pub fn new(id: MailId, backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        Self {
            id,
            backend,
            task_manager,

            vertical: ScrollbarState::default(),
            horizontal: ScrollbarState::default(),
            expected_overlay: None,
            scroll_action: None,
            throbber: ThrobberState::default(),
            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::Back),
                ("gg", Action::ScrollToTop),
                ("ge", Action::ScrollToBottom),
                ("<tab>", Action::OpenNextTab),
                ("<btab>", Action::OpenPreviousTab),
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("zl", Action::ScrollRight),
                ("zh", Action::ScrollLeft),
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
            Action::OpenTextTab => self.open_next_tab(),
            Action::OpenMarkdownTab => self.open_markdown_tab(),
            Action::OpenAttachmentsTab => self.open_attachments_tab(),
            Action::OpenNextTab => self.open_next_tab(),
            Action::OpenPreviousTab => self.open_previous_tab(),
            Action::OpenLogs => self.open_logs(),
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

impl SubModel for Model {
    fn request_if_missing(&self) {
        let mail = self.backend.get_mail(&self.id).unwrap();

        if mail.text_body.not_requested() {
            let id = self.id.clone();
            let backend = self.backend.clone();
            self.task_manager.spawn(async move {
                match backend.prefetch_mail_body(&id, MailBodyType::Text).await {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Couldn't fetch text-body of mail:\n{err}");
                    }
                }
            })
        }
    }
}

impl MailContentReader<Action> for Model {
    fn set_scroll_action(&mut self, scroll: super::ScrollAction) {
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
}
