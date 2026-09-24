use std::collections::HashMap;

use crate::{
    datasource::{
        ThreadCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailId, ThreadId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl ThreadCache for HashMapDataSource {
    async fn get_thread_state(&self) -> Option<&GetState> {
        self.threads_get_state.as_ref()
    }

    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<cache::GetBatchResult<HashMap<ThreadId, Vec<MailId>>, Vec<ThreadId>>> {
        let mut datas = HashMap::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.threads.get(id) {
                Some(thread_mails) => {
                    datas.insert(id.clone(), thread_mails.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: datas,
            missing,
        })
    }

    async fn upsert_thread(&mut self, id: ThreadId, mails: Vec<MailId>) -> Result<()> {
        self.threads.insert(id, mails);
        Ok(())
    }

    async fn set_thread_state(&mut self, new_state: GetState) -> Result<()> {
        self.threads_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_threads(&mut self, ids: &[ThreadId]) -> Result<()> {
        for id in ids {
            self.threads.remove(id);
        }
        Ok(())
    }
}
