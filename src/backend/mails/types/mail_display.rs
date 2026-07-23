use super::{MailAddress, MailData};
use crate::backend::mails::types::{MailId, MailKeyword};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadMarker {
    Single,
    Root,
    Child,
}

pub struct MailDisplay {
    pub id: MailId,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub preview: String,
    pub received_at: String,
    pub has_attachment: bool,
    pub keywords: HashSet<MailKeyword>,
    pub thread_marker: ThreadMarker,
}

impl MailDisplay {
    pub fn new(mail: &MailData, marker: ThreadMarker) -> Self {
        let id = mail.id.clone();
        let from = addresses_to_string(&mail.from);
        let to = addresses_to_string(&mail.to);
        let cc = addresses_to_string(&mail.cc);

        let subject = mail.subject.clone();
        let preview = mail.preview.clone();
        let received_at = mail
            .received_at
            .format("%a, %e %b %Y, %H:%M:%S")
            .to_string();
        let has_attachment = mail.has_attachment;
        let keywords = mail.keywords.clone();

        Self {
            id,
            from,
            to,
            cc,
            subject,
            preview,
            received_at,
            has_attachment,
            keywords,
            thread_marker: marker,
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
