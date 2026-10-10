use crate::{
    config::Username,
    types::{AccountId, CachedMail, MailId, MailProperty},
};

pub enum MessageRequest {
    GetMail {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
        properties: Vec<MailProperty>,
        callback: Box<dyn FnOnce(color_eyre::Result<CachedMail>) -> Vec<crate::ui::Message> + Send>,
    },
}

impl From<MessageRequest> for crate::ui::Message {
    fn from(msg: MessageRequest) -> Self {
        Self::PagerRequest(msg)
    }
}
