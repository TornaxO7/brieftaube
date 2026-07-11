use super::state::PaletteType;
use crate::ui::palette::Entry;
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
    #[strum(message = "Go back")]
    Back,

    #[strum(message = "Open logs")]
    OpenLogs,

    #[strum(message = "Select the next mail.")]
    SelectNextMail,
    #[strum(message = "Select the previous mail.")]
    SelectPreviousMail,
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
