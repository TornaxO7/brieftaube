use super::state::PaletteType;
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

    #[strum(message = "Quit the application")]
    Quit,
    #[strum(message = "Go back")]
    Back,

    #[strum(message = "Open logs")]
    OpenLogs,

    #[strum(message = "Compose a new mail.")]
    ComposeMail,

    #[strum(message = "Toggle mail selection.")]
    ToggleMailSelection,

    #[strum(
        message = "Mark the selected mails as unseen. If no mails are selected it will use the mail under the current cursor."
    )]
    MarkSelectedMailsAsUnseen,
    #[strum(
        message = "Mark the selected mails as seen. If no mails are selected it will use the mail under the current cursor."
    )]
    MarkSelectedMailsAsSeen,

    #[strum(message = "Unfold thread.")]
    UnfoldThread,
    #[strum(message = "Fold thread.")]
    FoldThread,

    #[strum(message = "Tries to fold the thread. If that doesn't work. Go back instead.")]
    FoldThreadOrGoBack,

    #[strum(message = "Navigate to the next (below) mail.")]
    NavigateToNextMail,
    #[strum(message = "Navigate to the previous (above) mail.")]
    NavigateToPreviousMail,
    #[strum(message = "Navigate to the top of the list.")]
    NavigateToTop,
    #[strum(message = "Navigate to the bottom of the list.")]
    NavigateToBottom,

    #[strum(message = "View the selected mail.")]
    ViewSelectedMail,
}

pub fn palette_options() -> Vec<Entry<PaletteType>> {
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
                value: PaletteType::Action(action),
                name,
                description,
            })
        })
        .collect()
}
