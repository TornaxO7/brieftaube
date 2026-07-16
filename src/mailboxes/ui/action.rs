use super::state::PaletteValue;
use crate::utils::ui::palette::Entry;
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

    #[strum(message = "Select next mailbox")]
    SelectNextMailbox,
    #[strum(message = "Select previous mailbox")]
    SelectPreviousMailbox,

    #[strum(message = "Open the selected mailbox.")]
    ActivateSelectedEntry,
    #[strum(message = "Go up one mailbox")]
    GoBack,

    #[strum(message = "Create a new mailbox")]
    CreateMailbox,
    #[strum(message = "Destroy the selected mailbox")]
    DestroySelectedMailbox,

    #[strum(message = "Set the sort order of the selected mailbox.")]
    SetSortOrder,
    #[strum(message = "Move the selected mailbox up")]
    MoveMailboxUp,
    #[strum(message = "Move the selected mailbox down")]
    MoveMailboxDown,

    #[strum(message = "Open logs")]
    OpenLogs,
    #[strum(message = "Quit the application")]
    Quit,
}

pub fn palette_options() -> Vec<Entry<PaletteValue>> {
    Action::iter()
        .filter_map(|action| {
            if let Some(is_intern) = action.get_bool("intern") {
                if is_intern {
                    return None;
                }
            }

            let name = action.to_string();
            let description = action.get_message().unwrap_or_default().to_string();

            Some(Entry {
                value: PaletteValue::Action(action),
                name,
                description,
            })
        })
        .collect()
}
