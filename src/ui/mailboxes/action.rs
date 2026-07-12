use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, EnumString};

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

    #[strum(message = "Open the selected mailbox")]
    OpenSelectedMailbox,

    #[strum(message = "Set the sort order of the selected mailbox.")]
    SetSortOrder,

    #[strum(message = "Quit the application")]
    Quit,
}
