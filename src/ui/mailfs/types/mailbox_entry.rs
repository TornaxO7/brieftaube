use crate::{
    types::{CachedMailbox, MailboxId},
    ui::mailfs::types::Selectable,
};

#[derive(Debug, Clone)]
pub struct MailboxEntry {
    pub id: MailboxId,
    pub name: String,
    pub unread_mails: usize,
    pub total_threads: usize,

    pub is_selected: bool,
}

impl From<CachedMailbox> for MailboxEntry {
    fn from(mailbox: CachedMailbox) -> Self {
        Self {
            id: mailbox.id,
            name: mailbox.name,
            unread_mails: mailbox.unread_mails,
            total_threads: mailbox.total_threads,

            is_selected: false,
        }
    }
}

impl Selectable for MailboxEntry {
    fn is_selected(&self) -> bool {
        self.is_selected
    }

    fn set_selection(&mut self, selected: bool) {
        self.is_selected = selected;
    }
}
