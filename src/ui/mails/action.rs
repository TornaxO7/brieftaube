use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, VariantArray};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    EnumIter,
    EnumMessage,
    EnumProperty,
    VariantArray,
    strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    #[strum(props(intern = true))]
    OpenCommandPalette,
    #[strum(message = "Quit the application")]
    Quit,

    #[strum(message = "Focus mail list.")]
    FocusMailList,
    #[strum(message = "Focus mailbox list.")]
    FocusMailBoxList,
    #[strum(message = "Focus mail preview.")]
    FocusPreview,
    #[strum(message = "Focus the right panel of the current panel.")]
    FocusRightPanel,
    #[strum(message = "Focus the left panel of the current panel.")]
    FocusLeftPanel,

    #[strum(message = "Select the next mailbox.")]
    SelectNextMailBox,
    #[strum(message = "Select the previous mailbox.")]
    SelectPreviousMailBox,

    #[strum(message = "Select the next mail.")]
    SelectNextMail,
    #[strum(message = "Select the previous mail.")]
    SelectPreviousMail,

    #[strum(message = "Create a new mail")]
    CreateNewMail,
}
