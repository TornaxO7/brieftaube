mod action;
use crate::{
    backend::{
        Backend,
        mails::types::{MailId, MailKeyword, MailUpdate},
    },
    palette,
    reader::{submodel::SubModel, types::MailDisplay},
    task_manager::TaskManager,
    utils::layer::{LayerCore, LayerModel},
};
use std::{rc::Rc, sync::Arc};
use tracing::error;

pub use super::submodel::attachments;
pub use super::submodel::markdown;
pub use super::submodel::metadata;
pub use super::submodel::text;
pub use action::Action;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Metadata,
    Text,
    Markdown,
    Attachments,
}

pub struct Model {
    id: MailId,
    backend: Arc<Backend>,

    pub mode: Mode,

    pub metadata: metadata::Model,
    pub text: text::Model,
    pub markdown: markdown::Model,
    pub attachments: attachments::Model,
}

impl Model {
    pub fn new(id: MailId, backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        let mode = match backend.config().reader.default_tab {
            crate::config::DefaultTab::Metadata => Mode::Metadata,
            crate::config::DefaultTab::Attachments => Mode::Attachments,
            crate::config::DefaultTab::Text => Mode::Text,
            crate::config::DefaultTab::Markdown => Mode::Markdown,
        };

        let id2 = id.clone();
        let backend2 = backend.clone();
        task_manager.spawn(async move {
            match backend2
                .update_mails(vec![MailUpdate {
                    id: id2,
                    patch_keywords: Some(vec![(MailKeyword::Seen, true)]),
                    ..Default::default()
                }])
                .await
            {
                Ok(()) => {}
                Err(err) => {
                    error!("Couldn't mark mail as \"seen\":\n{err}");
                }
            }
        });

        let text = {
            let text = text::Model::new(id.clone(), backend.clone(), task_manager.clone());
            if matches!(mode, Mode::Text) {
                text.request_if_missing();
            }
            text
        };

        let markdown = {
            let markdown = markdown::Model::new(id.clone(), backend.clone(), task_manager.clone());
            if matches!(mode, Mode::Markdown) {
                markdown.request_if_missing();
            }
            markdown
        };

        let attachments = {
            let attachments =
                attachments::Model::new(id.clone(), backend.clone(), task_manager.clone());
            if matches!(mode, Mode::Attachments) {
                attachments.request_if_missing();
            }
            attachments
        };

        Self {
            id: id.clone(),
            backend: backend.clone(),
            mode,

            metadata: metadata::Model::new(id.clone(), backend.clone(), task_manager.clone()),
            text,
            markdown,
            attachments,
        }
    }

    pub fn get_display_mail(&self) -> MailDisplay {
        let mail = self.backend.get_mail(&self.id).unwrap();
        MailDisplay::new(mail, self.backend.clone())
    }
}

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
        let action = match self.mode {
            Mode::Metadata => LayerCore::handle_event(&mut self.metadata, event, statusbar),
            Mode::Text => LayerCore::handle_event(&mut self.text, event, statusbar),
            Mode::Markdown => LayerCore::handle_event(&mut self.markdown, event, statusbar),
            Mode::Attachments => LayerCore::handle_event(&mut self.attachments, event, statusbar),
        };

        action.and_then(|action| self.apply_action(action))
    }
}

impl LayerModel<Action> for Model {
    fn apply_action(&mut self, action: Action) -> Option<crate::Action> {
        match action {
            Action::Quit => self.quit(),
            Action::OpenMetadataTab => self.open_metadata_tab(),
            Action::OpenTextTab => self.open_text_tab(),
            Action::OpenMarkdownTab => self.open_markdown_tab(),
            Action::OpenAttachmentsTab => self.open_attachments_tab(),
            Action::OpenNextTab => self.open_next_tab(),
            Action::OpenPreviousTab => self.open_previous_tab(),
            Action::OpenLogs => self.open_logs(),
            Action::Back => self.back(),

            Action::OpenPalette { entries } => self.open_palette(entries),

            Action::Metadata(action) => self
                .metadata
                .apply_action(action)
                .and_then(|a| self.apply_action(a)),
            Action::Text(action) => self
                .text
                .apply_action(action)
                .and_then(|a| self.apply_action(a)),
            Action::Markdown(action) => self
                .markdown
                .apply_action(action)
                .and_then(|a| self.apply_action(a)),
            Action::Attachments(action) => self
                .attachments
                .apply_action(action)
                .and_then(|a| self.apply_action(a)),
        }
    }

    fn handle_overlay<O: crate::utils::layer::LayerOverlay>(
        &mut self,
        overlay: O,
    ) -> Option<crate::Action> {
        let action = match self.mode {
            Mode::Metadata => self.metadata.handle_overlay(overlay),
            Mode::Text => self.text.handle_overlay(overlay),
            Mode::Markdown => self.markdown.handle_overlay(overlay),
            Mode::Attachments => self.attachments.handle_overlay(overlay),
        };

        action.and_then(|a| self.apply_action(a))
    }
}

impl Model {
    fn quit(&self) -> Option<crate::Action> {
        Some(crate::Action::Quit)
    }

    fn open_metadata_tab(&mut self) -> Option<crate::Action> {
        self.set_mode(Mode::Metadata);
        None
    }

    fn open_text_tab(&mut self) -> Option<crate::Action> {
        self.set_mode(Mode::Text);
        None
    }

    fn open_markdown_tab(&mut self) -> Option<crate::Action> {
        self.set_mode(Mode::Markdown);
        None
    }

    fn open_attachments_tab(&mut self) -> Option<crate::Action> {
        self.set_mode(Mode::Attachments);
        None
    }

    fn open_next_tab(&mut self) -> Option<crate::Action> {
        let next = match self.mode {
            Mode::Metadata => Mode::Text,
            Mode::Text => Mode::Markdown,
            Mode::Markdown => Mode::Attachments,
            Mode::Attachments => Mode::Metadata,
        };

        self.set_mode(next);
        None
    }

    fn open_previous_tab(&mut self) -> Option<crate::Action> {
        let previous = match self.mode {
            Mode::Metadata => Mode::Attachments,
            Mode::Text => Mode::Metadata,
            Mode::Markdown => Mode::Text,
            Mode::Attachments => Mode::Markdown,
        };

        self.set_mode(previous);
        None
    }

    fn open_logs(&self) -> Option<crate::Action> {
        Some(crate::Action::OpenLogViewer)
    }

    fn back(&self) -> Option<crate::Action> {
        Some(crate::Action::Back)
    }

    fn open_palette(&self, entries: Vec<palette::PaletteEntry>) -> Option<crate::Action> {
        Some(crate::Action::OpenPalette { entries })
    }
}

impl Model {
    fn set_mode(&mut self, viewer: Mode) {
        self.mode = viewer;

        match viewer {
            Mode::Metadata => {}
            Mode::Attachments => {}
            Mode::Text => self.text.request_if_missing(),
            Mode::Markdown => self.markdown.request_if_missing(),
        }
    }
}
