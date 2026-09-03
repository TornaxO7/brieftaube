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

    async fn upsert_thread<MailIds>(
        &mut self,
        id: &ThreadId,
        mails: MailIds,
    ) -> Result<(), Self::Error>
    where
        MailIds: IntoIterator<Item = MailId>,
    {
        self.threads.insert(id.clone(), mails.into_iter().collect());
        Ok(())
    }

    async fn set_thread_state(&mut self, new_state: GetState) -> Result<(), Self::Error> {
        self.threads_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_thread(&mut self, id: &ThreadId) -> Result<(), Self::Error> {
        self.threads.remove(id);
        Ok(())
    }
}
