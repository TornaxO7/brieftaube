use crate::{
    config,
    datasource::types::QueryWindow,
    types::{AccountId, CachedMail, MailId, MailProperty, MailboxId, ParentMailboxId, ThreadId},
};

pub enum MessageRequest {
    GetAccountsOf(config::UserConfig),

    GetMail {
        username: config::Username,
        account_id: AccountId,
        mail_id: MailId,
        properties: Vec<MailProperty>,
        callback:
            Box<dyn FnOnce(color_eyre::Result<CachedMail>) -> crate::ui::mailfs::Message + Send>,
    },

    InitMailbox {
        username: config::Username,
        account_id: AccountId,
        mailbox_id: ParentMailboxId,
        /// How many mails should be requested at the same time when getting the mailbox data
        max_init_mails: usize,
        mail_properties: Vec<MailProperty>,
    },

    // GetChildMailboxes {
    //     username: config::Username,
    //     account_id: AccountId,
    //     parent_id: ParentMailboxId,
    // },
    QueryMails {
        username: config::Username,
        account_id: AccountId,
        mailbox: MailboxId,

        window: QueryWindow,
        mail_properties: Vec<MailProperty>,
    },
    GetThreadMails {
        username: config::Username,
        account_id: AccountId,
        thread_id: ThreadId,
        mail_properties: Vec<MailProperty>,
    },
    // GetMailPreview {
    //     username: config::Username,
    //     account_id: AccountId,
    //     mail_id: MailId,
    // },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::MailfsRequest(msg)
    }
}
