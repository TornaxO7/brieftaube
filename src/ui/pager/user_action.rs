use crate::ui::palette::PaletteEntry;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, EnumString, IntoEnumIterator};

// TODO: We list all of thos actions.
// Check during runtime if the selected action is
// valid in this case.
// If not: Log to statusbar!
//
// Why: Users can immediately see what the can do in the pager.
// They don't need to navigate to the attachments
// to see actions which can be used to download attachments.
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

    #[strum(message = "Navigate half page down.")]
    NavigateHalfPageDown,
    #[strum(message = "Navigate half page up.")]
    NavigateHalfPageUp,

    #[strum(message = "Open the displayed body in your editor.")]
    OpenBodyInEditor,

    #[strum(message = "Focus the next tab.")]
    FocusNextTab,

    #[strum(message = "Go back.")]
    Back,
    #[strum(message = "Quit the application.")]
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
