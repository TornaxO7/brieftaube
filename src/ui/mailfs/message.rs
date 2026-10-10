use crate::{
    config::Username,
    datasource::types::QueryWindow,
    types::{
        AccountData, AccountId, CachedMail, InitMailboxData, MailId, MailboxId, ParentMailboxId,
        ThreadId,
    },
    ui::mailfs::{types::MailPreview, user_action::UserAction},
};
use crossterm::event::Event;
use ratatui::layout::Size;

#[derive(Debug)]
pub enum Message {
    Event(Event),
    UserAction(UserAction),

    SetUserAccounts {
        username: Username,
        accounts: color_eyre::Result<Vec<AccountData>>,
    },

    InitMailbox {
        username: Username,
        account_id: AccountId,
        mailbox_id: ParentMailboxId,

        data: color_eyre::Result<InitMailboxData>,
    },

    // SetChildMailboxes {
    //     username: Username,
    //     account_id: AccountId,
    //     parent_id: ParentMailboxId,
    //     child_mailboxes: color_eyre::Result<Vec<MailboxData>>,
    // },
    SetRootMails {
        username: Username,
        account_id: AccountId,
        mailbox: MailboxId,

        window: QueryWindow,

        result: color_eyre::Result<Vec<CachedMail>>,
    },
    SetThreadMails {
        username: Username,
        account_id: AccountId,
        thread_id: ThreadId,

        thread_mails: color_eyre::Result<Vec<CachedMail>>,
    },
    SetMailPreview {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,

        preview: color_eyre::Result<MailPreview>,
    },

    SelectedPaletteEntry(String),

    SetColumnAreaSize(Size),
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Mailfs(msg)
    }
}
