use crate::backend::{Backend, MailData, ThreadId};

impl Backend {
    pub fn get_or_request_thread_mails(&self, id: &ThreadId) -> Option<Vec<MailData>> {
        let thread_mails = self.get_thread_mails(id);

        if thread_mails.is_none() {
            self.get_or_request_thread_mails(id);
        }

        thread_mails
    }

    pub fn get_thread_mails(&self, id: &ThreadId) -> Option<Vec<MailData>> {
        let store = self.store.lock().unwrap();

        let thread_mail_ids = store
            .threads
            .get_mails(id)
            .expect("Thread has already been requested.");

        thread_mail_ids
            .iter()
            .map(|mail_id| store.mails.get(mail_id).cloned())
            .collect()
    }

    fn request_thread_mails(&self, id: &ThreadId) {
        let thread_mail_ids = {
            let store = self.store.lock().unwrap();
            store.threads.get_mails(&id).unwrap().to_vec()
        };

        self.request_mails(&thread_mail_ids);
    }
}
