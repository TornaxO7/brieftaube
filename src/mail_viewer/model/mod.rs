mod action;

use crate::{
    backend::{
        Backend, MailBodyType, MailData,
        mails::types::{MailId, MailKeyword, MailUpdate},
    },
    palette,
    task_manager::TaskManager,
    utils::layer::{LayerCore, LayerModel},
};
use std::{rc::Rc, sync::Arc};
use tracing::error;

pub use super::submodel::attachments::AttachmentsViewer;
pub use super::submodel::markdown::MarkdownViewer;
pub use super::submodel::metadata::MetadataViewer;
pub use super::submodel::text::TextViewer;
pub use action::Action;

#[derive(Debug, Clone, Copy)]
pub enum Viewer {
    Metadata,
    Text,
    Markdown,
    Attachments,
}

// #[derive(Debug, Clone, Copy)]
// pub enum ScrollAction {
//     ScrollDown(usize),
//     ScrollUp(usize),
//     ScrollHalfPageDown,
//     ScrollHalfPageUp,
//     ScrollHalfPageRight,
//     ScrollHalfPageLeft,
//     ScrollLeft(usize),
//     ScrollRight(usize),
//     SetTop,
//     SetBottom,
// }

pub struct Model {
    id: MailId,
    backend: Arc<Backend>,
    task_manager: Rc<TaskManager>,

    pub viewer: Viewer,

    pub metadata: MetadataViewer,
    pub text: TextViewer,
    pub markdown: MarkdownViewer,
    pub attachments: AttachmentsViewer,
    // /// Contains the scrolling action for the current, selected viewer.
    // /// Since we don't know the height and width of the area where each viewer
    // /// gets rendered to, we have to apply the scroll action _later_ during the rendering...
    // pub scroll_action: Option<ScrollAction>,
}

impl Model {
    pub fn new(id: MailId, backend: Arc<Backend>, task_manager: Rc<TaskManager>) -> Self {
        let selected_viewer = match backend.config().mail_viewer.default_tab {
            crate::config::DefaultTab::Metadata => Viewer::Metadata,
            crate::config::DefaultTab::Attachments => Viewer::Attachments,
            crate::config::DefaultTab::Text => Viewer::Text,
            crate::config::DefaultTab::Markdown => Viewer::Markdown,
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

        let model = Self {
            id,
            backend,
            task_manager,

            viewer: selected_viewer,

            metadata: MetadataViewer::new(),
            text: TextViewer::new(),
            markdown: MarkdownViewer::new(),
            attachments: AttachmentsViewer::new(),
        };

        model.request_body_if_absent();

        model
    }

    pub fn get_mail(&self) -> MailData {
        self.backend.get_mail(&self.id).unwrap()
    }

    fn request_body_if_absent(&self) {
        let mail = self.backend.get_mail(&self.id).unwrap();

        let ty = match self.viewer {
            Viewer::Metadata | Viewer::Attachments => return,
            Viewer::Markdown => MailBodyType::Html,
            Viewer::Text => MailBodyType::Text,
        };

        let body_is_missing = match ty {
            MailBodyType::Text => mail.text_body.is_none(),
            MailBodyType::Html => mail.html_body.is_none(),
        };

        if body_is_missing {
            let id = self.id.clone();
            let b = self.backend.clone();
            self.task_manager.spawn(async move {
                match b.prefetch_mail_body(&id, ty).await {
                    Ok(()) => {}
                    Err(err) => {
                        let ty_name = match ty {
                            MailBodyType::Text => "text",
                            MailBodyType::Html => "html",
                        };

                        error!("Couldn't fetch {ty_name}-body of mail:\n{err}");
                    }
                }
            });
        }
    }
}

impl LayerCore for Model {
    fn handle_event(
        &mut self,
        event: crossterm::event::Event,
        statusbar: &mut crate::statusbar::Model,
    ) -> Option<crate::Action> {
        let action = match self.viewer {
            Viewer::Metadata => LayerCore::handle_event(&mut self.metadata, event, statusbar),
            Viewer::Text => LayerCore::handle_event(&mut self.text, event, statusbar),
            Viewer::Markdown => LayerCore::handle_event(&mut self.markdown, event, statusbar),
            Viewer::Attachments => LayerCore::handle_event(&mut self.attachments, event, statusbar),
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
        let action = match self.viewer {
            Viewer::Metadata => self.metadata.handle_overlay(overlay),
            Viewer::Text => self.text.handle_overlay(overlay),
            Viewer::Markdown => self.markdown.handle_overlay(overlay),
            Viewer::Attachments => self.attachments.handle_overlay(overlay),
        };

        action.and_then(|a| self.apply_action(a))
    }
}

impl Model {
    fn quit(&self) -> Option<crate::Action> {
        Some(crate::Action::Quit)
    }

    fn open_metadata_tab(&mut self) -> Option<crate::Action> {
        self.set_viewer(Viewer::Metadata);
        None
    }

    fn open_text_tab(&mut self) -> Option<crate::Action> {
        self.set_viewer(Viewer::Text);
        None
    }

    fn open_markdown_tab(&mut self) -> Option<crate::Action> {
        self.set_viewer(Viewer::Markdown);
        None
    }

    fn open_attachments_tab(&mut self) -> Option<crate::Action> {
        self.set_viewer(Viewer::Attachments);
        None
    }

    fn open_next_tab(&mut self) -> Option<crate::Action> {
        let next = match self.viewer {
            Viewer::Metadata => Viewer::Text,
            Viewer::Text => Viewer::Markdown,
            Viewer::Markdown => Viewer::Attachments,
            Viewer::Attachments => Viewer::Metadata,
        };

        self.set_viewer(next);
        None
    }

    fn open_previous_tab(&mut self) -> Option<crate::Action> {
        let previous = match self.viewer {
            Viewer::Metadata => Viewer::Attachments,
            Viewer::Text => Viewer::Metadata,
            Viewer::Markdown => Viewer::Text,
            Viewer::Attachments => Viewer::Markdown,
        };

        self.set_viewer(previous);
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
    fn set_viewer(&mut self, viewer: Viewer) {
        self.viewer = viewer;

        match viewer {
            Viewer::Metadata | Viewer::Attachments => {}
            Viewer::Text | Viewer::Markdown => self.request_body_if_absent(),
        }
    }
}
