mod action;
mod widget;

use super::SubModel;
use crate::{
    backend::{Backend, MailId},
    task_manager::TaskManager,
    utils::{
        keybindmanager::KeybindManager,
        layer::{LayerCore, LayerModel, LayerModelDefaultHandleEvent},
    },
};
use arboard::Clipboard;
use ratatui::widgets::TableState;
use std::{collections::HashMap, rc::Rc, str::FromStr, sync::Arc};
use tracing::{error, warn};

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

    id: MailId,
    backend: Arc<Backend>,
    task_manager: Rc<TaskManager>,

    expected_overlay: Option<ExpectedOverlay>,
    navigate: Option<Navigate>,
}

impl Model {
    pub fn new(id: MailId, backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        Self {
            state: TableState::new(),
            expected_overlay: None,
            navigate: None,

            id,
            backend,
            task_manager,

            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::Back),
                ("gg", Action::NavigateToTop),
                ("ge", Action::NavigateToBottom),
                ("<tab>", Action::OpenNextTab),
                ("<btab>", Action::OpenPreviousTab),
                ("j", Action::NavigateDown),
                ("k", Action::NavigateUp),
                (":", Action::OpenCommandPalette),
                ("q", Action::Quit),
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
            Action::DownloadAttachment => self.download_attachment(),
            Action::CopyPathToClipboard => self.copy_path_to_clipboard(),
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

impl SubModel for Model {
    fn request_if_missing(&self) {}
}

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

    fn download_attachment(&self) -> Option<super::Action> {
        let attachment_idx = self.state.selected()?;
        let mail = self.backend.get_mail(&self.id)?;

        let attachment = mail
            .attachments
            .get()?
            .loaded()?
            .get(attachment_idx)?
            .clone();

        let backend = self.backend.clone();
        self.task_manager.spawn(async move {
            match backend.download_attachment(&attachment).await {
                Ok(()) => {}
                Err(err) => {
                    error!("Couldn't download attachment: {err}");
                }
            }
        });
        None
    }

    fn copy_path_to_clipboard(&self) -> Option<super::Action> {
        let mail = self.backend.get_mail(&self.id).unwrap();
        let attachment_idx = self.state.selected()?;

        let Some(attachments) = mail.attachments.get() else {
            warn!("Mail attachments haven't been loaded yet.");
            return None;
        };

        let Some(attachments) = attachments.loaded() else {
            warn!("Mail attachments haven't arrived yet.");
            return None;
        };

        let attachment = attachments.get(attachment_idx).unwrap();

        let Some(path) = self.backend.get_attachment_path(attachment) else {
            error!("Attachment isn't downloaded yet. Please download it first.");
            return None;
        };

        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(err) => {
                error!("Clipboards aren't supported:\n{err}");
                return None;
            }
        };

        if let Err(err) = clipboard.set_text(path.to_string_lossy()) {
            error!("Couldn't set clipboard:\n{err}");
            return None;
        }

        None
    }
}
