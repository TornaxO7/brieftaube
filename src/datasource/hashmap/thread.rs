use crate::{
    datasource::{
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
    ) -> Result<cache::GetBatchResult<Vec<(ThreadId, Vec<MailId>)>, Vec<ThreadId>>, Self::Error>
    {
        let inner = self.inner.read().unwrap();

        let mut cached_threads = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match inner.threads.get(id) {
                Some(thread_mails) => cached_threads.push((id.clone(), thread_mails.clone())),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_threads,
            missing,
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
