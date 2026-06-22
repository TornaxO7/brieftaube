use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, EnumString, VariantArray};

use crate::ui::command_palette::CommandPaletteEntry;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    EnumIter,
    EnumMessage,
    EnumProperty,
    EnumString,
    VariantArray,
    strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    #[strum(props(intern = true))]
    OpenCommandPalette,
    #[strum(props(intern = true))]
    CloseCommandPalette,

    #[strum(message = "Quit the application")]
    Quit,

    #[strum(message = "Select the next mailbox.")]
    SelectNextMailbox,
    #[strum(message = "Select the previous mailbox.")]
    SelectPreviousMailbox,

    #[strum(message = "Select the next mail.")]
    SelectNextMail,
    #[strum(message = "Select the previous mail.")]
    SelectPreviousMail,
    // #[strum(message = "Create a new mail")]
    // CreateNewMail,
    // #[strum(message = "Open mail in the pager.")]
    // OpenMailInPager,
}

impl CommandPaletteEntry for Action {}
