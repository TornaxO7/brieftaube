use crate::types::{CachedMailbox, MailboxId};

#[derive(Debug, Clone)]
pub struct MailboxEntry {
    pub id: MailboxId,
    pub name: String,
    pub unread_mails: usize,
    pub total_threads: usize,
}

impl From<CachedMailbox> for MailboxEntry {
    fn from(mailbox: CachedMailbox) -> Self {
        Self {
            id: mailbox.id,
            name: mailbox.name,
            unread_mails: mailbox.unread_mails,
            total_threads: mailbox.total_threads,
        }
    }
}
