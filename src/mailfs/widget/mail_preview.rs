use crate::backend::mails::types::{MailAddress, MailData, MailKeyword};
use std::collections::HashSet;

pub struct MailPreview {
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub preview: String,
    pub received_at: String,
    pub keywords: HashSet<MailKeyword>,
}

impl From<&MailData> for MailPreview {
    fn from(mail: &MailData) -> Self {
        let from = addresses_to_string(&mail.from);
        let to = addresses_to_string(&mail.to);
        let cc = addresses_to_string(&mail.cc);

        let subject = mail.subject.clone();
        let preview = mail.preview.clone();
        let received_at = mail
            .received_at
            .format("%a, %e %b %Y, %H:%M:%S")
            .to_string();
        let keywords = mail.keywords.clone();

        Self {
            from,
            to,
            cc,
            subject,
            preview,
            received_at,
            keywords,
        }
    }
}

pub fn addresses_to_string(addresses: &[MailAddress]) -> String {
    let mut iterator = addresses.iter();
    let first = iterator
        .next()
        .map(|addr| format!("{}", addr))
        .unwrap_or(String::new());

    iterator.fold(first, |acc, addr| format!("{acc}, {}", addr.to_string()))
}
