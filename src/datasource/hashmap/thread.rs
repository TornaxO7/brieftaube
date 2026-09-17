use crate::{
    datasource::{ThreadCache, hashmap::HashMapDataSource, types::GetState},
    types::{MailId, ThreadId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl ThreadCache for HashMapDataSource {
    async fn get_thread_state(&self) -> Option<&GetState> {
        self.threads_get_state.as_ref()
    }

    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Vec<MailId>>> {
        Ok(self.threads.get(id).cloned())
    }

    async fn upsert_thread(&mut self, id: ThreadId, mails: Vec<MailId>) -> Result<()> {
        self.threads.insert(id, mails);
        Ok(())
    }

    async fn set_thread_state(&mut self, new_state: GetState) -> Result<()> {
        self.threads_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_thread(&mut self, id: &ThreadId) -> Result<()> {
        self.threads.remove(id);
        Ok(())
    }
}
