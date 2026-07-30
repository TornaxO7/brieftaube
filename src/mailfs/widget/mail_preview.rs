use crate::backend::mails::types::{MailData, addresses_to_string};

#[derive(Debug)]
pub struct MailPreview {
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub preview: String,
    pub received_at: String,
}

impl From<MailData> for MailPreview {
    fn from(mail: MailData) -> Self {
        let from = addresses_to_string(&mail.from);
        let to = addresses_to_string(&mail.to);
        let cc = addresses_to_string(&mail.cc);

        let subject = mail.subject.clone();
        let preview = mail.preview.clone();
        let received_at = mail
            .received_at
            .format("%a, %e %b %Y, %H:%M:%S")
            .to_string();
        // let has_attachment = mail.has_attachment;
        // let keywords = mail.keywords.clone();

        Self {
            from,
            to,
            cc,
            subject,
            preview,
            received_at,
        }
    }
}
