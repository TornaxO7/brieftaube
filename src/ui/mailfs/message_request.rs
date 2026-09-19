use crate::{
    config,
    datasource::types::QueryWindow,
    repository,
    types::{AccountId, MailboxId, ParentMailboxId, ThreadId},
};

pub enum MessageRequest {
    GetAccountsOf(config::Username),
    RepositoryCommand {
        user: config::Username,
        command: repository::Command,
    },
    GetChildMailboxes {
        account_id: AccountId,
        parent: ParentMailboxId,
    },
    QueryMails {
        account_id: AccountId,
        mailbox: MailboxId,
        window: QueryWindow,
    },
    GetThreadMails {
        account_id: AccountId,
        thread: ThreadId,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::MailfsRequest(msg)
    }
}
