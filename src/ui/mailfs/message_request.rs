use crate::{
    config,
    types::{AccountId, MailId, ParentMailboxId, ThreadId},
};

pub enum MessageRequest {
    GetAccountsOf(config::UserConfig),

    InitMailbox {
        username: config::Username,
        account_id: AccountId,
        mailbox_id: ParentMailboxId,
        /// How many mails should be requested at the same time when getting the mailbox data
        max_init_mails: usize,
    },

    // GetChildMailboxes {
    //     username: config::Username,
    //     account_id: AccountId,
    //     parent_id: ParentMailboxId,
    // },
    // QueryMails {
    //     username: config::Username,
    //     account_id: AccountId,
    //     mailbox: MailboxId,

    //     window: QueryWindow,
    //     calculate_total: bool,
    // },
    GetThreadMails {
        username: config::Username,
        account_id: AccountId,
        thread_id: ThreadId,
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
