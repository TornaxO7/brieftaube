use crate::{
    config::Username,
    types::{AccountId, BlobId, CachedMail, MailId, MailProperty},
    ui::repository_manager::RepositoryManager,
};

pub enum MessageRequest {
    GetMail {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
        properties: Vec<MailProperty>,
        callback: Box<dyn FnOnce(color_eyre::Result<CachedMail>) -> Vec<crate::ui::Message> + Send>,
    },
    GetBlob {
        username: Username,
        account_id: AccountId,
        blob_id: BlobId,

        callback: fn(color_eyre::Result<Vec<u8>>) -> Vec<crate::ui::Message>,
    },

    EditDraftMail {
        username: Username,
        account_id: AccountId,

        new_draft: CachedMail,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::PagerRequest(msg)
    }
}

pub fn handle_request(
    msg: MessageRequest,
    manager: &mut RepositoryManager,
) -> Vec<crate::ui::Message> {
    match msg {
        MessageRequest::GetMail {
            username,
            account_id,
            mail_id,
            properties,
            callback,
        } => {
            manager.with_handler(&username, async move |handler| {
                callback(handler.get_mail(account_id, mail_id, properties).await)
            });
            vec![]
        }
        MessageRequest::EditDraftMail {
            username,
            account_id,
            new_draft,
        } => {
            manager.with_handler(&username, async move |handler| {
                vec![
                    super::Message::SetMail(handler.edit_draft_mail(account_id, new_draft).await)
                        .into(),
                ]
            });
            vec![]
        }
        MessageRequest::GetBlob {
            username,
            account_id,
            blob_id,
            callback,
        } => {
            manager.with_handler(&username, async move |handler| {
                callback(handler.get_blob(account_id, blob_id).await)
            });
            vec![]
        }
    }
}
