use ratatui::widgets::{ScrollbarState, TableState};

use crate::mail_viewer::{
    state::{AttachmentViewer, MarkdownViewer, MetadataViewer, ScrollAction, TextViewer},
    types::MailDisplay,
};

pub struct RenderData<'a> {
    pub viewer_state: ViewerState<'a>,
    pub mail: MailDisplay,
    pub scroll_queue: &'a mut Option<ScrollAction>,
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
