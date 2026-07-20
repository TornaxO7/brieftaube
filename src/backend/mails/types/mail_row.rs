use super::{MailData, MailKeyword, mail_preview::addresses_to_string};
use std::collections::HashSet;

pub struct MailRow {
    pub from: String,
    pub subject: String,
    pub has_attachment: bool,
    pub keywords: HashSet<MailKeyword>,
    pub selected: bool,
    pub received_at: String,
    pub thread_marker: ThreadMarker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThreadMarker {
    #[default]
    None,
    Branch,
    Last,
}

impl From<&MailData> for MailRow {
    fn from(mail: &MailData) -> Self {
        Self {
            from: addresses_to_string(&mail.from),
            subject: mail.subject.clone(),
            has_attachment: mail.has_attachment,
            keywords: mail.keywords.clone(),
            selected: false,
            received_at: mail
                .received_at
                .format("%a, %e %b %Y, %H:%M:%S")
                .to_string(),
            thread_marker: ThreadMarker::None,
        }
    }
}
