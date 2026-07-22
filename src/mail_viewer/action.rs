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
    #[strum(message = "Go back.")]
    Back,

    #[strum(message = "Scroll down")]
    ScrollDown,
    #[strum(message = "Scroll up")]
    ScrollUp,
    #[strum(message = "Scroll left")]
    ScrollLeft,
    #[strum(message = "Scroll right")]
    ScrollRight,
    #[strum(message = "Scroll to the top.")]
    ScrollToTop,
    #[strum(message = "Scroll to the top.")]
    ScrollToBottom,

    #[strum(message = "Scroll half page down.")]
    ScrollHalfPageDown,
    #[strum(message = "Scroll half page up.")]
    ScrollHalfPageUp,

    #[strum(message = "Scroll half page to the right.")]
    ScrollHalfPageRight,
    #[strum(message = "Scroll half page to the left.")]
    ScrollHalfPageLeft,

    #[strum(message = "Display the mail as text.")]
    OpenTextTab,
    #[strum(message = "Display the mail of html-text as markdown.")]
    OpenMarkdownTab,
    #[strum(message = "Open log viewer")]
    OpenLogs,
    #[strum(message = "Open html mail in browser")]
    OpenMailInBrowser,
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
