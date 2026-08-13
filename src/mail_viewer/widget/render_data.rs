use crate::{
    backend::MailBodyType,
    mail_viewer::{
        Model,
        model::{
            AttachmentViewer, MarkdownViewer, MetadataViewer, ScrollAction, TextViewer, Viewer,
        },
        types::MailDisplay,
    },
};
use ratatui::widgets::{ScrollbarState, TableState};
use tracing::error;

pub struct RenderData<'a> {
    pub viewer_state: ViewerState<'a>,
    pub mail: MailDisplay,
    pub scroll_action: &'a mut Option<ScrollAction>,
}

impl<'a> RenderData<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        let viewer_state = match model.selected_viewer {
            Viewer::Metadata => ViewerState::from(&mut model.metadata_viewer),
            Viewer::Text => {
                let mail = model.backend.get_mail(&model.id).unwrap();
                let text_body_not_fetched_yet = mail.text_body.is_none();
                if text_body_not_fetched_yet {
                    let id = model.id.clone();
                    let b = model.backend.clone();
                    model.task_manager.spawn(async move {
                        match b.prefetch_mail_body(&id, MailBodyType::Text).await {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't fetch text-body of mail:\n{err}");
                            }
                        }
                    });
                }
                ViewerState::from(&mut model.text_viewer)
            }
            Viewer::Markdown => {
                let mail = model.backend.get_mail(&model.id).unwrap();
                let html_body_not_fetched_yet = mail.html_body.is_none();
                if html_body_not_fetched_yet {
                    let id = model.id.clone();
                    let b = model.backend.clone();
                    model.task_manager.spawn(async move {
                        match b.prefetch_mail_body(&id, MailBodyType::Html).await {
                            Ok(()) => {}
                            Err(err) => {
                                error!("Couldn't fetch html-body of mail:\n{err}");
                            }
                        }
                    });
                }
                ViewerState::from(&mut model.markdown_viewer)
            }
            Viewer::Attachments => ViewerState::from(&mut model.attachment_viewer),
        };

        let mail = model.backend.get_mail(&model.id).unwrap();

        Self {
            viewer_state,
            mail: MailDisplay::from(mail),
            scroll_action: &mut model.scroll_action,
        }
    }
}

pub enum ViewerState<'a> {
    Metadata(&'a mut TableState),
    Text {
        vertical: &'a mut ScrollbarState,
        horizontal: &'a mut ScrollbarState,
    },
    Markdown {
        vertical: &'a mut ScrollbarState,
        horizontal: &'a mut ScrollbarState,
    },
    Attachments(&'a mut TableState),
}

impl<'a> From<&'a mut Model> for ViewerState<'a> {
    fn from(model: &'a mut Model) -> Self {
        match model.selected_viewer {
            Viewer::Metadata => Self::from(&mut model.metadata_viewer),
            Viewer::Text => Self::from(&mut model.text_viewer),
            Viewer::Markdown => Self::from(&mut model.markdown_viewer),
            Viewer::Attachments => Self::from(&mut model.attachment_viewer),
        }
    }
}

impl<'a> From<&'a mut MetadataViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut MetadataViewer) -> Self {
        Self::Metadata(&mut viewer.state)
    }
}

impl<'a> From<&'a mut TextViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut TextViewer) -> Self {
        Self::Text {
            vertical: &mut viewer.vertical,
            horizontal: &mut viewer.horizontal,
        }
    }
}

impl<'a> From<&'a mut MarkdownViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut MarkdownViewer) -> Self {
        Self::Markdown {
            vertical: &mut viewer.vertical,
            horizontal: &mut viewer.horizontal,
        }
    }
}

impl<'a> From<&'a mut AttachmentViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut AttachmentViewer) -> Self {
        Self::Attachments(&mut viewer.state)
    }
}
