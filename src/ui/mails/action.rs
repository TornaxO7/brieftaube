use crate::ui::command_palette::Entry;
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
    #[strum(props(intern = true))]
    CloseCommandPalette,

    #[strum(message = "Quit the application")]
    Quit,

    #[strum(message = "Select mailbox by using the palette.")]
    OpenMailboxPalette,
    #[strum(message = "Select the given mailbox", props(intern = true))]
    SelectMailbox(String),
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

impl Action {
    pub fn palette_options() -> Vec<Entry> {
        Self::iter()
            .filter_map(|action| {
                if let Some(is_intern) = action.get_bool("intern") {
                    if is_intern {
                        return None;
                    }
                }

                let name = action.to_string();
                let description = action.get_message().unwrap_or_default().to_string();

                Some(Entry { name, description })
            })
            .collect()
    }
}
