use crate::types::{CachedMail, MailId, MailKeyword, MailProperty, ThreadId};
use chrono::{DateTime, Local};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailEntry {
    pub id: MailId,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub thread_id: ThreadId,
}

impl MailEntry {
    pub const MAIL_PROPERTIES: [MailProperty; 6] = [
        MailProperty::Id,
        MailProperty::Keywords,
        MailProperty::Subject,
        MailProperty::ReceivedAt,
        MailProperty::HasAttachment,
        MailProperty::ThreadId,
    ];
}

impl From<CachedMail> for MailEntry {
    fn from(cached_mail: CachedMail) -> Self {
        let id = cached_mail.id;
        let keywords = cached_mail.keywords.into_inner().unwrap();
        let subject = cached_mail.subject.into_inner();
        let received_at = cached_mail.received_at.into_inner().unwrap();
        let has_attachment = cached_mail.has_attachment.into_inner().unwrap();
        let thread_id = cached_mail.thread_id.into_inner().unwrap();

        Self {
            id,
            keywords,
            subject,
            received_at,
            has_attachment,
            thread_id,
        }
    }
}
