use super::{MailboxData, MailboxId};

#[derive(Clone, Default)]
pub struct Node {
    mailbox_owner: Option<MailboxId>,
    mailboxes: Vec<MailboxData>,
}

impl Node {
    pub fn new(mailbox_owner: Option<MailboxId>) -> Self {
        Self {
            mailbox_owner,
            mailboxes: Vec::new(),
        }
    }

    pub fn add_mailbox(&mut self, mailbox: MailboxData) {
        let idx = self
            .mailboxes
            .partition_point(|m| m.sort_order < mailbox.sort_order);

        self.mailboxes.insert(idx, mailbox);
    }

    pub fn is_root_node(&self) -> bool {
        self.mailbox_owner.is_none()
    }

    pub fn contains_mailbox_name(&self, name: &str) -> bool {
        self.mailboxes.iter().any(|mailbox| mailbox.name == name)
    }

    pub fn contains_mailbox(&self, id: &MailboxId) -> bool {
        self.get_mailbox(id).is_some()
    }

    pub fn get_mailbox(&self, id: &MailboxId) -> Option<&MailboxData> {
        self.mailboxes
            .iter()
            .find(|mailbox| mailbox.id == id.as_str())
    }

    pub fn get_mailbox_mut(&mut self, id: &MailboxId) -> Option<&mut MailboxData> {
        self.mailboxes
            .iter_mut()
            .find(|mailbox| mailbox.id == id.as_str())
    }

    pub fn remove_mailbox(&mut self, id: &MailboxId) {
        self.mailboxes.retain(|mailbox| mailbox.id != id);
    }
}
