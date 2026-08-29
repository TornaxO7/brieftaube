use crate::{
    datasources::{
        MailboxCache, MailboxDataSource,
        hashmap::HashMapDataSource,
        types::{GetResult, GetState, SetResult},
    },
    types::{MailboxData, MailboxId, MailboxUpdate},
};

impl MailboxDataSource for HashMapDataSource {
    async fn get_mailboxes(
        &self,
        ids: Option<&[MailboxId]>,
    ) -> Result<GetResult<Vec<MailboxData>>, Self::Error> {
        todo!()
    }

    async fn create_mailbox(
        &self,
        new: crate::types::MailboxNew,
    ) -> Result<SetResult<MailboxData>, Self::Error> {
        todo!()
    }

    async fn update_mailbox(&self, update: MailboxUpdate) -> Result<SetResult<()>, Self::Error> {
        todo!()
    }

    async fn destroy_mailbox(&self, id: &MailboxId) -> Result<SetResult<()>, Self::Error> {
        todo!()
    }
}

impl MailboxCache for HashMapDataSource {
    async fn upsert_mailboxes(
        &self,
        mailboxes: Vec<MailboxData>,
        state: GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn evict_mailboxes(
        &self,
        ids: &[MailboxId],
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
