use crate::backend::mails::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, addresses_to_string,
};

pub struct FullMailDisplay {
    pub id: MailId,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub received_at: String,
    pub has_attachment: bool,

    pub html_body: Option<MailDataHtmlBody>,
    pub text_body: Option<MailDataTextBody>,
    pub attachments: Option<Vec<MailDataAttachment>>,
}

impl From<MailData> for FullMailDisplay {
    fn from(mail: MailData) -> Self {
        Self {
            id: mail.id,
            from: addresses_to_string(&mail.from),
            to: addresses_to_string(&mail.to),
            cc: addresses_to_string(&mail.cc),
            subject: mail.subject,
            received_at: mail.received_at.format("%A, %d %B %Y %T").to_string(),
            has_attachment: mail.has_attachment,

            html_body: mail.html_body,
            text_body: mail.text_body,
            attachments: mail.attachments,
        }
    }
}
