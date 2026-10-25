use crate::types::{CachedMailbox, MailboxId};

#[derive(Debug, Clone)]
pub struct MailboxEntry {
    pub id: MailboxId,
    pub name: String,
    pub unread_mails: usize,
    pub total_threads: usize,
    pub sort_order: u32,
}

impl From<CachedMailbox> for MailboxEntry {
    fn from(mailbox: CachedMailbox) -> Self {
        Self {
            id: mailbox.id,
            name: mailbox.name.into_inner().unwrap(),
            unread_mails: mailbox.unread_mails.into_inner().unwrap(),
            total_threads: mailbox.total_threads.into_inner().unwrap(),
            sort_order: mailbox.sort_order.into_inner().unwrap(),
        }
    }
}
