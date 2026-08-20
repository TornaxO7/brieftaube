use crate::backend::{Backend, MailboxData, MailboxId};

impl Backend {
    pub fn get_mailbox(&self, id: &MailboxId) -> MailboxData {
        self.get_mailboxes(&[id.clone()])[0].clone()
    }

    pub fn get_mailboxes<Ids: AsRef<[MailboxId]>>(&self, ids: Ids) -> Vec<MailboxData> {
        let store = self.store.lock().unwrap();

        ids.as_ref()
            .into_iter()
            .map(|id| store.mailbox.get_data(id).clone())
            .collect()
    }
}
