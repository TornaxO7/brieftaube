use crate::{
    config::{UserConfig, Username},
    datasource::types::{MutateAttribute, QueryWindow},
    types::{
        AccountId, CachedMail, MailId, MailKeyword, MailProperty, MailboxId, ParentMailboxId,
        ThreadId,
    },
    ui::{mailfs::Message, repository_manager::RepositoryManager},
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

pub fn handle_request(
    msg: MessageRequest,
    manager: &mut RepositoryManager,
) -> Vec<crate::ui::Message> {
    match msg {
        MessageRequest::GetAccountsOf(user_config) => {
            manager.load_user(user_config);
            vec![]
        }
        MessageRequest::InitMailbox {
            username,
            account_id,
            mailbox_id,
            max_init_mails: amount_initial_mails,
            mail_properties,
        } => {
            manager.with_handler(&username.clone(), async move |handler| {
                vec![
                    Message::InitMailbox {
                        username,
                        account_id: account_id.clone(),
                        mailbox_id: mailbox_id.clone(),

                        data: handler
                            .get_init_mailbox(
                                account_id,
                                mailbox_id,
                                amount_initial_mails,
                                mail_properties,
                            )
                            .await,
                    }
                    .into(),
                ]
            });
            vec![]
        }
        MessageRequest::GetMail {
            username,
            account_id,
            mail_id,
            properties,
            callback,
        } => {
            manager.with_handler(&username.clone(), async move |handler| {
                vec![callback(handler.get_mail(account_id, mail_id, properties).await).into()]
            });
            vec![]
        }
        MessageRequest::QueryMails {
            username,
            account_id,
            mailbox,
            window,
            mail_properties,
        } => {
            manager.with_handler(&username.clone(), async move |handler| {
                vec![
                    Message::SetRootMails {
                        username,
                        account_id: account_id.clone(),
                        mailbox: mailbox.clone(),
                        window: window.clone(),
                        result: handler
                            .query_mails(account_id, mailbox, window, mail_properties)
                            .await,
                    }
                    .into(),
                ]
            });
            vec![]
        }
        MessageRequest::GetThreadMails {
            username,
            account_id,
            thread_id,
            mail_properties,
        } => {
            manager.with_handler(&username.clone(), async move |handler| {
                vec![
                    Message::SetThreadMails {
                        username,
                        account_id: account_id.clone(),
                        thread_id: thread_id.clone(),
                        thread_mails: handler
                            .get_thread_mails(account_id, thread_id, mail_properties)
                            .await,
                    }
                    .into(),
                ]
            });
            vec![]
        }

        MessageRequest::MutateMail {
            username,
            account_id,
            ids,
            mailbox_ids,
            keywords,
        } => {
            manager.with_handler(&username.clone(), async move |handler| {
                match handler
                    .mutate_mails(account_id, ids, mailbox_ids, keywords)
                    .await
                {
                    Ok(()) => vec![Message::ReloadColumnEntries.into()],
                    Err(_err) => vec![],
                }
            });
            vec![]
        }
    }
}
