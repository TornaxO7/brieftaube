use crate::backend::mails::types::{MailData, MailDataRest, MailId, addresses_to_string};

pub struct FullMailDisplay {
    pub id: MailId,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub received_at: String,
    pub has_attachment: bool,
    pub rest: MailDataRest,
}

impl From<&MailData> for FullMailDisplay {
    fn from(mail: &MailData) -> Self {
        let rest = mail.rest.clone().unwrap();

        Self {
            id: mail.id.clone(),
            from: addresses_to_string(&mail.from),
            to: addresses_to_string(&mail.to),
            cc: addresses_to_string(&mail.cc),
            subject: mail.subject.clone(),
            received_at: mail.received_at.format("%A, %d %B %Y %T").to_string(),
            has_attachment: mail.has_attachment,
            rest,
        }
    }
}
