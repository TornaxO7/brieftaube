use crate::palette;

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
}
