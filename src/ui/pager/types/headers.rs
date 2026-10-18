use crate::types::{CachedMail, MailProperty};

#[derive(Debug)]
pub struct MailHeaders {
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub received_at: Option<String>,
}

impl MailHeaders {
    pub const MAX_AMOUNT_HEADERS: usize = 5;
    pub const LONGEST_HEADER_LENGTH: usize = "Received at:".len();

    pub const MAIL_PROPERTIES: [MailProperty; 5] = [
        MailProperty::From,
        MailProperty::To,
        MailProperty::Cc,
        MailProperty::Subject,
        MailProperty::ReceivedAt,
    ];
}

impl From<&CachedMail> for MailHeaders {
    fn from(cached_mail: &CachedMail) -> Self {
        Self {
            from: cached_mail
                .from
                .get()
                .map(|addresses| addresses.to_string()),
            to: cached_mail.to.get().map(|addresses| addresses.to_string()),
            cc: cached_mail.cc.get().map(|addresses| addresses.to_string()),
            bcc: cached_mail.bcc.get().map(|addresses| addresses.to_string()),
            subject: cached_mail.subject.get().cloned(),
            received_at: cached_mail
                .received_at
                .get()
                .map(|date| date.format("%c").to_string()),
        }
    }
}
