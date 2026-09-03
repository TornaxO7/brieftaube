use crate::{
    datasource::{ThreadCache, hashmap::HashMapDataSource, types::GetState},
    types::{MailId, ThreadId},
};

impl ThreadCache for HashMapDataSource {
    async fn get_thread_state(&self) -> Option<&GetState> {
        self.threads_get_state.as_ref()
    }

    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Vec<MailId>>, Self::Error> {
        Ok(self.threads.get(id).cloned())
    }

    // async fn upsert_thread(
    //     &mut self,
    //     id: &ThreadId,
    //     mails: &[MailData],
    //     new_get_mail_state: GetState,
    //     new_get_thread_state: GetState,
    // ) -> Result<(), Self::Error> {
    //     let mail_ids = mails.iter().map(|mail| mail.id.clone()).collect();
    //     self.threads.insert(id.clone(), mail_ids);

    //     for mail in mails {
    //         let mail_id = mail.id.clone();
    //         self.mails.insert(mail_id, mail.clone());
    //     }

    //     self.mail_get_state = Some(new_get_mail_state);
    //     self.threads_get_state = Some(new_get_thread_state);
    //     Ok(())
    // }

    async fn set_thread_state(&mut self, new_state: GetState) -> Result<(), Self::Error> {
        self.threads_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_thread(&mut self, id: &ThreadId) -> Result<(), Self::Error> {
        self.threads.remove(id);
        Ok(())
    }
}
