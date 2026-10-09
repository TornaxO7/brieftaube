use super::MailKeyword;
use crate::types::{MailDto, MailId, MailboxId, ThreadId};
use chrono::{DateTime, Local};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailDataCore {
    pub id: MailId,
    pub keywords: HashSet<MailKeyword>,
    pub subject: Option<String>,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: Vec<MailboxId>,
    pub thread_id: ThreadId,
}

impl From<MailDto> for MailDataCore {
    fn from(cached_mail: MailDto) -> Self {
        let stage0 = cached_mail.core;

        Self {
            id: stage0.id,
            keywords: stage0.keywords,
            subject: stage0.subject,
            received_at: stage0.received_at.with_timezone(&Local),
            has_attachment: stage0.has_attachment,
            mailbox_ids: stage0.mailbox_ids,
            thread_id: stage0.thread_id,
        }
    }
}
