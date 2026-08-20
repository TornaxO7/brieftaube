use crate::backend::{Backend, MailboxData, MailboxId};

impl Backend {
    pub fn get_mailbox(&self, id: &MailboxId) -> MailboxData {
        let store = self.store.lock().unwrap();
        store.mailbox.get_data(id).clone()
    }
}
