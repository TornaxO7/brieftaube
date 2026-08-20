use crate::backend::{Backend, MailId, ThreadId};

impl Backend {
    pub fn get_thread(&self, id: &ThreadId) -> Vec<MailId> {
        let mut store = self.store.lock().unwrap();
        store.threads.get_mails(id).to_vec()
    }
}
