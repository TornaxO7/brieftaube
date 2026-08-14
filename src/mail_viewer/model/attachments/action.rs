use crate::palette::PaletteEntry;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, EnumString, IntoEnumIterator};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    EnumIter,
    EnumMessage,
    EnumProperty,
    EnumString,
    strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    #[strum(props(intern = true))]
    OpenCommandPalette,

    #[strum(message = "Quit the application")]
    Quit,
    #[strum(message = "Go back.")]
    Back,

    #[strum(message = "Scroll down")]
    NavigateDown,
    #[strum(message = "Navigate up")]
    NavigateUp,
    #[strum(message = "Navigate to the top.")]
    NavigateToTop,
    #[strum(message = "Navigate to the top.")]
    NavigateToBottom,

    #[strum(message = "Navigate half page down.")]
    NavigateHalfPageDown,
    #[strum(message = "Navigate half page up.")]
    NavigateHalfPageUp,

    #[strum(message = "Navigate half page to the right.")]
    NavigateHalfPageRight,
    #[strum(message = "Navigate half page to the left.")]
    NavigateHalfPageLeft,

    #[strum(message = "Display metadata of mail.")]
    OpenMetadataTab,
    #[strum(message = "Display the mail as text.")]
    OpenTextTab,
    #[strum(message = "Display the mail of html-text as markdown.")]
    OpenMarkdownTab,
    #[strum(message = "Display attachments of the mail.")]
    OpenAttachmentsTab,

    #[strum(message = "Open next tab (to the right)")]
    OpenNextTab,
    #[strum(message = "Open previous tab (to the left)")]
    OpenPreviousTab,

    #[strum(message = "Open log viewer")]
    OpenLogs,
}

pub fn palette_options() -> Vec<PaletteEntry> {
    Action::iter()
        .filter_map(|action| {
            if let Some(is_intern) = action.get_bool("intern") {
                if is_intern {
                    return None;
                }
            }

            let name = action.to_string();
            let description = action.get_message().unwrap_or_default().to_string();

            Some(PaletteEntry { name, description })
        })
        .collect()
}
