use crate::datasources::{MailboxCache, MailboxDataSource, hashmap::HashMapDataSource};

impl MailboxDataSource for HashMapDataSource {
    async fn get_mailboxes(
        &self,
    ) -> Result<crate::datasources::types::GetResult<Vec<crate::types::MailboxData>>, Self::Error>
    {
        todo!()
    }

    async fn create_mailbox(
        &self,
        new: crate::types::MailboxNew,
    ) -> Result<crate::datasources::types::SetResult<crate::types::MailboxData>, Self::Error> {
        todo!()
    }

    async fn update_mailbox(
        &self,
        update: crate::types::MailboxUpdate,
    ) -> Result<crate::datasources::types::SetResult<()>, Self::Error> {
        todo!()
    }

    async fn destroy_mailbox(
        &self,
        id: &crate::types::MailboxId,
    ) -> Result<crate::datasources::types::SetResult<()>, Self::Error> {
        todo!()
    }
}

impl MailboxCache for HashMapDataSource {
    async fn upsert_mailboxes(
        &self,
        mailboxes: &[crate::types::MailboxData],
        state: crate::datasources::types::GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn evict_mailboxes(
        &self,
        ids: &[crate::types::MailboxId],
        new_state: crate::datasources::types::GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
