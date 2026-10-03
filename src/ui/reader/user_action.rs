use crate::ui::palette::PaletteEntry;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, EnumString, IntoEnumIterator};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    EnumString,
    EnumIter,
    EnumMessage,
    EnumProperty,
    strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserAction {
    #[strum(props(intern = true))]
    OpenCommandPalette,

    #[strum(message = "Open html body")]
    OpenHtmlBody,
    #[strum(message = "Open text body")]
    OpenTextBody,

    #[strum(message = "Navigate to the next (below) mailbox.")]
    NavigateDown,
    #[strum(message = "Navigate to the previous (above) mailbox.")]
    NavigateUp,
    #[strum(message = "Navigate to the top of the list.")]
    NavigateToTop,
    #[strum(message = "Navigate to the bottom of the list.")]
    NavigateToBottom,

    NavigateHalfPageDown,
    NavigateHalfPageUp,

    #[strum(message = "")]
    FocusNextTab,

    #[strum(message = "Go back")]
    Back,
    #[strum(message = "Quit the application")]
    Quit,
}

impl UserAction {
    pub fn palette_options() -> Vec<PaletteEntry> {
        Self::iter()
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
}
