use crate::types::{CachedMail, MailId, MailKeyword, MailProperty, MailboxId, ThreadId};
use chrono::{DateTime, Local};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailColumnEntry {
    pub id: MailId,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: ThreadId,
}

impl MailColumnEntry {
    pub const MAIL_PROPERTIES: [MailProperty; 7] = [
        MailProperty::Id,
        MailProperty::Keywords,
        MailProperty::Subject,
        MailProperty::ReceivedAt,
        MailProperty::HasAttachment,
        MailProperty::MailboxIds,
        MailProperty::ThreadId,
    ];
}

impl From<CachedMail> for MailColumnEntry {
    fn from(cached_mail: CachedMail) -> Self {
        let id = cached_mail.id;
        let keywords = cached_mail.keywords.into_inner().unwrap();
        let subject = cached_mail.subject.into_inner();
        let received_at = cached_mail.received_at.into_inner().unwrap();
        let has_attachment = cached_mail.has_attachment.into_inner().unwrap();
        let mailbox_ids = cached_mail.mailbox_ids.into_inner().unwrap();
        let thread_id = cached_mail.thread_id.into_inner().unwrap();

        Self {
            id,
            keywords,
            subject,
            received_at,
            has_attachment,
            mailbox_ids,
            thread_id,
        }
    }
}
