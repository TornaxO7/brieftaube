use crate::{
    config,
    datasource::types::QueryWindow,
    repository,
    types::{MailboxId, ParentMailboxId, ThreadId},
};

pub enum MessageRequest {
    GetAccountsOf(config::Username),
    RepositoryCommand {
        user: config::Username,
        command: repository::Command,
    },
    GetChildMailboxes {
        parent: ParentMailboxId,
    },
    QueryMails {
        mailbox: MailboxId,
        window: QueryWindow,
    },
    GetThreadMails {
        thread: ThreadId,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::MailfsRequest(msg)
    }
}
