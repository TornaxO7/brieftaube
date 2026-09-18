use crate::{
    config::Username,
    types::AccountData,
    ui::{Loadable, mailfs::user_action::UserAction},
};
use crossterm::event::Event;

pub enum Message {
    Event(Event),
    UserAction(UserAction),

    SetUserAccounts {
        username: Username,
        accounts: Loadable<Vec<AccountData>>,
    },

    SelectedPaletteEntry(String),
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Mailfs(msg)
    }
}
