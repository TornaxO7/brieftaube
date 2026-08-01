use ratatui::widgets::{ScrollbarState, TableState};

use crate::mail_viewer::{
    state::{AttachmentViewer, HeadersViewer, MarkdownViewer, ScrollAction, TextViewer},
    types::FullMailDisplay,
};

pub struct RenderData<'a> {
    pub viewer_state: ViewerState<'a>,
    pub mail: FullMailDisplay,
    pub scroll_queue: &'a mut Option<ScrollAction>,
}

pub enum ViewerState<'a> {
    Headers(&'a mut TableState),
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

impl<'a> From<&'a mut HeadersViewer> for ViewerState<'a> {
    fn from(viewer: &'a mut HeadersViewer) -> Self {
        Self::Headers(&mut viewer.state)
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
        Self::Headers(&mut viewer.state)
    }
}
