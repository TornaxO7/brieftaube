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
pub enum UserAction {
    #[strum(props(intern = true))]
    OpenCommandPalette,

    #[strum(message = "Navigate to the next (below) mailbox.")]
    NavigateDown,
    #[strum(message = "Navigate to the previous (above) mailbox.")]
    NavigateUp,
    #[strum(message = "Navigate to the top of the list.")]
    NavigateToTop,
    #[strum(message = "Navigate to the bottom of the list.")]
    NavigateToBottom,

    #[strum(message = "Open the selected mailbox.")]
    NavigateRight,
    #[strum(message = "Open the selected mailbox.")]
    NavigateLeft,

    #[strum(message = "Open the parent mailbox.")]
    NavigateToParent,

    #[strum(message = "Toggle entry selection")]
    SelectEntryToggle,
    #[strum(message = "Mark the current selection as 'cut'.")]
    CutSelectedEntries,

    #[strum(message = "Paste the entries which are marked as 'cut'.")]
    PasteSelectedEntries,

    #[strum(message = "Mark the given mail as seen.")]
    MarkMailAsSeen,
    #[strum(message = "Mark the given mail as unseen.")]
    MarkMailAsUnseen,

    #[strum(message = "Move the selected mailbox up")]
    MoveMailboxUp,
    #[strum(message = "Move the selected mailbox down")]
    MoveMailboxDown,

    #[strum(message = "Create a new mailbox in the current mailbox.")]
    CreateMailbox,
    #[strum(message = "Remove mailbox only, if it's empty.")]
    RemoveMailbox,

    #[strum(message = "Open logs")]
    OpenLogs,
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
