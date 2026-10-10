use crate::types::{CachedAttachment, CachedMail, MailAddresses, MailProperty};
use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub struct MailPreview {
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub subject: Option<String>,
    pub preview: String,
    pub attachments: Vec<CachedAttachment>,
    pub received_at: DateTime<Local>,
}

impl MailPreview {
    pub const MAIL_PROPERTIES: [MailProperty; 6] = [
        MailProperty::From,
        MailProperty::To,
        MailProperty::Cc,
        MailProperty::Bcc,
        MailProperty::Preview,
        MailProperty::Attachments,
    ];
}

impl From<CachedMail> for MailPreview {
    fn from(cached_mail: CachedMail) -> Self {
        let from = cached_mail.from.into_inner();
        let to = cached_mail.to.into_inner();
        let cc = cached_mail.cc.into_inner();
        let bcc = cached_mail.bcc.into_inner();
        let subject = cached_mail.subject.into_inner();
        let preview = cached_mail.preview.into_inner().unwrap();
        let attachments = cached_mail.attachments.into_inner().unwrap();
        let received_at = cached_mail.received_at.into_inner().unwrap();

        Self {
            from,
            to,
            cc,
            bcc,
            subject,
            preview,
            attachments,
            received_at,
        }
    }
}
