use crate::{
    config::Username,
    datasource::types::{Patches, QueryWindow},
    types::{
        AccountId, CachedMail, CachedMailbox, MailId, MailProperty, MailboxId, ParentMailboxId,
        ThreadId,
    },
    ui::{
        mailfs::{Message, types::UserEntry},
        repository_manager::RepositoryManager,
    },
};
use tracing::error;

pub enum MessageRequest {
    GetAccountsOf(UserEntry),

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
        mail_window: QueryWindow,
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

    PatchMailAndMailboxes {
        username: Username,
        account_id: AccountId,

        patches: Patches,
    },

    CreateMailbox {
        username: Username,
        account_id: AccountId,

        new_mailbox: CachedMailbox,
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
        MessageRequest::GetAccountsOf(user) => {
            manager.load_user(user.username);
            vec![]
        }
        MessageRequest::InitMailbox {
            username,
            account_id,
            mailbox_id,
            mail_window,
            mail_properties,
        } => {
            manager.with_handler(&username, async move |handler| {
                vec![
                    Message::InitMailbox {
                        account_id: account_id.clone(),
                        mailbox_id: mailbox_id.clone(),

                        data: handler
                            .get_init_mailbox(account_id, mailbox_id, mail_window, mail_properties)
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
            manager.with_handler(&username, async move |handler| {
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
            manager.with_handler(&username, async move |handler| {
                vec![
                    Message::SetRootMails {
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
            manager.with_handler(&username, async move |handler| {
                vec![
                    Message::SetThreadMails {
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

        MessageRequest::PatchMailAndMailboxes {
            username,
            account_id,
            patches,
        } => {
            manager.with_handler(&username, async move |handler| {
                match handler.patch_mails_and_mailboxes(account_id, patches).await {
                    Ok(()) => vec![Message::Reload.into()],
                    Err(err) => {
                        error!("Couldn't patch mail: {}", err);
                        vec![]
                    }
                }
            });
            vec![]
        }
        MessageRequest::CreateMailbox {
            username,
            account_id,
            new_mailbox,
        } => {
            manager.with_handler(&username, async move |handler| {
                match handler.create_mailbox(account_id, new_mailbox).await {
                    Ok(_new_mailbox) => vec![Message::Reload.into()],
                    Err(err) => {
                        error!("Couldn't create mailbox: {}", err);
                        vec![]
                    }
                }
            });

            vec![]
        }
    }
}
