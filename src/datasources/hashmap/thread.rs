use crate::{
    datasources::{
        ThreadCache, ThreadDataSource,
        hashmap::HashMapDataSource,
        types::{GetResult, GetState},
    },
    types::{MailId, ThreadId},
};

impl ThreadDataSource for HashMapDataSource {
    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<GetResult<Vec<Option<Vec<MailId>>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let threads = ids
            .iter()
            .map(|id| inner.threads.get(id).cloned())
            .collect();

        Ok(GetResult {
            value: threads,
            state: inner.threads_get_state.clone(),
        })
    }
}

// TODO: add `upsert_threads`
impl ThreadCache for HashMapDataSource {
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
