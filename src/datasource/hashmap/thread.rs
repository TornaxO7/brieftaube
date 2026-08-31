use crate::{
    datasource::{
        ThreadCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailData, ThreadId},
};

impl ThreadCache for HashMapDataSource {
    async fn get_thread(
        &self,
        id: &ThreadId,
    ) -> Result<cache::GetOneResult<Option<Vec<MailData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let Some(thread_mail_ids) = inner.threads.get(id) else {
            return Ok(cache::GetOneResult {
                value: None,
                state: inner.threads_get_state.clone(),
            });
        };

        let mut thread_mails = Vec::new();
        for thread_mail_id in thread_mail_ids {
            let thread_mail = inner
                .mails
                .get(&thread_mail_id)
                .expect("Thread mails are inserted with its content!");

            thread_mails.push(thread_mail.clone());
        }

        Ok(cache::GetOneResult {
            value: Some(thread_mails),
            state: inner.threads_get_state.clone(),
        })
    }

    async fn upsert_thread(
        &self,
        id: &ThreadId,
        mails: &[MailData],
        new_get_mail_state: GetState,
        new_get_thread_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        let mail_ids = mails.iter().map(|mail| mail.id.clone()).collect();
        inner.threads.insert(id.clone(), mail_ids);

        for mail in mails {
            let mail_id = mail.id.clone();
            inner.mails.insert(mail_id, mail.clone());
        }

        inner.mail_get_state = Some(new_get_mail_state);
        inner.threads_get_state = Some(new_get_thread_state);
        Ok(())
    }

    async fn evict_thread(&self, id: &ThreadId, new_state: GetState) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();
        if let Some(thread_mail_ids) = inner.threads.remove(id) {
            for thread_mail_id in thread_mail_ids {
                inner.mails.remove(&thread_mail_id);
            }
        }

        inner.threads_get_state = Some(new_state);
        Ok(())
    }
}
