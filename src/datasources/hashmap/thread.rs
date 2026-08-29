use crate::datasources::{ThreadCache, ThreadDataSource, hashmap::HashMapDataSource};

impl ThreadDataSource for HashMapDataSource {
    async fn get_thread(
        &self,
        id: &crate::types::ThreadId,
    ) -> Result<crate::datasources::types::GetResult<Vec<crate::types::MailId>>, Self::Error> {
        todo!()
    }
}

impl ThreadCache for HashMapDataSource {
    async fn upsert_thread(
        &self,
        id: &crate::types::ThreadId,
        mails: &[crate::types::MailId],
        new_state: crate::datasources::types::GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn evict_thread(
        &self,
        id: &crate::types::ThreadId,
        new_state: crate::datasources::types::GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
