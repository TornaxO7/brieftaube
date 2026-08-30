use crate::{
    repository::datasource::{
        ThreadCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailId, ThreadId},
};

impl ThreadCache for HashMapDataSource {
    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<cache::GetResult<Vec<Option<Vec<MailId>>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let threads = ids
            .iter()
            .map(|id| inner.threads.get(id).cloned())
            .collect();

        Ok(cache::GetResult {
            value: threads,
            state: inner.threads_get_state.clone(),
        })
    }

    async fn upsert_thread(
        &self,
        id: &ThreadId,
        mails: &[MailId],
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();
        inner.threads.insert(id.clone(), mails.to_vec());
        inner.threads_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_thread(&self, id: &ThreadId, new_state: GetState) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();
        inner.threads.remove(id);
        inner.threads_get_state = Some(new_state);
        Ok(())
    }
}
