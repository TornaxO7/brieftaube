use crate::{
    config::{UserConfig, Username},
    datasource::types::{MutateAttribute, QueryWindow},
    types::{
        AccountId, CachedMail, MailId, MailKeyword, MailProperty, MailboxId, ParentMailboxId,
        ThreadId,
    },
};

pub enum MessageRequest {
    GetAccountsOf(UserConfig),

    GetMail {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
        properties: Vec<MailProperty>,
        callback:
            Box<dyn FnOnce(color_eyre::Result<CachedMail>) -> crate::ui::mailfs::Message + Send>,
    },

    InitMailbox {
        username: Username,
        account_id: AccountId,
        mailbox_id: ParentMailboxId,
        /// How many mails should be requested at the same time when getting the mailbox data
        max_init_mails: usize,
        mail_properties: Vec<MailProperty>,
    },

    QueryMails {
        username: Username,
        account_id: AccountId,
        mailbox: MailboxId,

        window: QueryWindow,
        mail_properties: Vec<MailProperty>,
    },
    GetThreadMails {
        username: Username,
        account_id: AccountId,
        thread_id: ThreadId,
        mail_properties: Vec<MailProperty>,
    },
    MutateMail {
        username: Username,
        account_id: AccountId,

        ids: Vec<MailId>,
        mailbox_ids: Option<MutateAttribute<MailboxId>>,
        keywords: Option<MutateAttribute<MailKeyword>>,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::MailfsRequest(msg)
    }
}
