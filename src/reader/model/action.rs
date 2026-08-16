use crate::{
    palette,
    reader::model::{attachments, markdown, metadata, text},
};

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

    OpenPalette { entries: Vec<palette::PaletteEntry> },

    Metadata(metadata::Action),
    Text(text::Action),
    Markdown(markdown::Action),
    Attachments(attachments::Action),
}
