use crate::{
    config::Username,
    datasource::types::QueryWindow,
    types::{
        AccountData, AccountId, CachedMail, InitMailboxData, MailId, MailboxId, ParentMailboxId,
        ThreadId,
    },
    ui::mailfs::types::MailPreview,
};
use crossterm::event::Event;
use ratatui::layout::Size;

#[derive(Debug)]
pub enum Message {
    Reload,

    Event(Event),

    SetUserAccounts {
        username: Username,
        accounts: color_eyre::Result<Vec<AccountData>>,
    },

    InitMailbox {
        account_id: AccountId,
        mailbox_id: ParentMailboxId,

        data: color_eyre::Result<InitMailboxData>,
    },

    SetRootMails {
        account_id: AccountId,
        mailbox: MailboxId,

        window: QueryWindow,
        result: color_eyre::Result<Vec<CachedMail>>,
    },
    SetThreadMails {
        account_id: AccountId,
        thread_id: ThreadId,

        thread_mails: color_eyre::Result<Vec<CachedMail>>,
    },
    SetMailPreview {
        account_id: AccountId,
        mail_id: MailId,

        preview: color_eyre::Result<MailPreview>,
    },

    SelectedPaletteEntry(String),

    SetColumnAreaSize(Size),

    CreateMailboxStep(CreateMailboxStep),
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Mailfs(msg)
    }
}

#[derive(Debug)]
pub enum CreateMailboxStep {
    PromptName(ParentMailboxId),
    CreateWithName(String, ParentMailboxId),
}
