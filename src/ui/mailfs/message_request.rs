use crate::{
    config,
    datasource::types::QueryWindow,
    types::{AccountId, MailId, MailboxId, ParentMailboxId, ThreadId},
};

pub enum MessageRequest {
    GetAccountsOf(config::UserConfig),
    GetChildMailboxes {
        username: config::Username,
        account_id: AccountId,
        parent_id: ParentMailboxId,
    },
    QueryMails {
        username: config::Username,
        account_id: AccountId,
        mailbox: MailboxId,

        window: QueryWindow,
        calculate_total: bool,
    },
    GetThreadMails {
        username: config::Username,
        account_id: AccountId,
        thread: ThreadId,
    },
    GetMailPreview {
        username: config::Username,
        account_id: AccountId,
        mail_id: MailId,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::MailfsRequest(msg)
    }
}
