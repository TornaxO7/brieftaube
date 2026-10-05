use crate::{
    config::Username,
    types::{AccountId, MailId},
};

#[derive(Debug)]
pub enum MessageRequest {
    GetHeaders {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    },
    GetTextBody {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,

        after_fetching: Vec<crate::ui::Message>,
    },
    GetHtmlBody {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,

        after_fetching: Vec<crate::ui::Message>,
    },
    GetAttachments {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::PagerRequest(msg)
    }
}
