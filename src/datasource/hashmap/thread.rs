use crate::{
    datasource::{ThreadCache, hashmap::HashMapDataSource, types::GetState},
    types::{MailData, ThreadId},
};

impl ThreadCache for HashMapDataSource {
    async fn get_thread_state(&self) -> Option<&GetState> {
        self.threads_get_state.as_ref()
    }

    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Vec<MailData>>, Self::Error> {
        let Some(thread_mail_ids) = self.threads.get(id) else {
            return Ok(None);
        };

        let mut thread_mails = Vec::new();
        for thread_mail_id in thread_mail_ids {
            let thread_mail = self
                .mails
                .get(&thread_mail_id)
                .expect("Thread mails are inserted with its content!");

            thread_mails.push(thread_mail.clone());
        }

        Ok(Some(thread_mails))
    }

    async fn upsert_thread(
        &mut self,
        id: &ThreadId,
        mails: &[MailData],
        new_get_mail_state: GetState,
        new_get_thread_state: GetState,
    ) -> Result<(), Self::Error> {
        let mail_ids = mails.iter().map(|mail| mail.id.clone()).collect();
        self.threads.insert(id.clone(), mail_ids);

        for mail in mails {
            let mail_id = mail.id.clone();
            self.mails.insert(mail_id, mail.clone());
        }

        self.mail_get_state = Some(new_get_mail_state);
        self.threads_get_state = Some(new_get_thread_state);
        Ok(())
    }

    async fn evict_thread(
        &mut self,
        id: &ThreadId,
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        if let Some(thread_mail_ids) = self.threads.remove(id) {
            for thread_mail_id in thread_mail_ids {
                self.mails.remove(&thread_mail_id);
            }
        }

        self.threads_get_state = Some(new_state);
        Ok(())
    }
}
