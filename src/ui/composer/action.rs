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

    #[strum(message = "Focus 'Mail' panel")]
    FocusMailPanel,
    #[strum(message = "Focus 'Attachments' panel")]
    FocusAttachmentsPanel,
}
