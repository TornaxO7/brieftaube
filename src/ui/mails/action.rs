use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumMessage, EnumProperty, VariantArray};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, EnumIter, EnumMessage, EnumProperty, VariantArray,
)]
#[serde(rename_all = "snake_case")]
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
}
