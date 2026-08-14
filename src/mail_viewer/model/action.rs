use crate::mail_viewer::model::{attachments, markdown, metadata, text};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Back,
    OpenMetadataTab,
    OpenTextTab,
    OpenMarkdownTab,
    OpenAttachmentsTab,
    OpenNextTab,
    OpenPreviousTab,
    OpenLogs,

    Metadata(metadata::Action),
    Text(text::Action),
    Markdown(markdown::Action),
    Attachments(attachments::Action),
}
