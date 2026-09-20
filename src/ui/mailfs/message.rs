use crate::{
    config::Username,
    types::{AccountData, AccountId, MailboxData, ParentMailboxId},
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
    SetChildMailboxes {
        username: Username,
        account_id: AccountId,
        parent_id: ParentMailboxId,
        child_mailboxes: color_eyre::Result<Vec<MailboxData>>,
    },

    SelectedPaletteEntry(String),
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Mailfs(msg)
    }
}
