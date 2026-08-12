use crate::backend::{Backend, MailData, ThreadId};

impl Backend {
    pub async fn get_or_request_thread_mails(
        &self,
        id: &ThreadId,
    ) -> Result<Vec<MailData>, jmap_client::Error> {
        match self.get_thread_mails(id) {
            Some(thread_mails) => Ok(thread_mails),
            None => {
                let thread_mail_ids = {
                    let store = self.store.lock().unwrap();
                    store.threads.get_mails(&id).unwrap().to_vec()
                };

                self.get_or_request_mails(&thread_mail_ids).await
            }
        }
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
}
