use crate::mail_viewer::{
    Model,
    model::{AttachmentsViewer, MarkdownViewer, MetadataViewer, ScrollAction, TextViewer, Viewer},
    types::MailDisplay,
};
use ratatui::widgets::{ScrollbarState, TableState};

pub struct RenderData<'a> {
    pub viewer_state: ViewerState<'a>,
    pub mail: MailDisplay,
    pub scroll_action: &'a mut Option<ScrollAction>,
}

impl<'a> RenderData<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        let mail = model.get_mail();

        let viewer_state = match model.viewer {
            Viewer::Metadata => ViewerState::from(&mut model.metadata),
            Viewer::Text => ViewerState::from(&mut model.text),
            Viewer::Markdown => ViewerState::from(&mut model.markdown),
            Viewer::Attachments => ViewerState::from(&mut model.attachments),
        };

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
        match model.viewer {
            Viewer::Metadata => Self::from(&mut model.metadata),
            Viewer::Text => Self::from(&mut model.text),
            Viewer::Markdown => Self::from(&mut model.markdown),
            Viewer::Attachments => Self::from(&mut model.attachments),
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

impl<'a> From<&'a mut AttachmentsViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut AttachmentsViewer) -> Self {
        Self::Attachments(&mut viewer.state)
    }
}
