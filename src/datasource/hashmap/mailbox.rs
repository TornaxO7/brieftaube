use crate::{
    datasource::{
        MailboxCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailboxData, MailboxId, ParentMailboxId},
};

impl MailboxCache for HashMapDataSource {
    fn get_mailbox_state(&self) -> Option<GetState> {
        let inner = self.inner.read().unwrap();
        inner.mailboxes_get_state.clone()
    }

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetBatchResult<Vec<MailboxData>, Vec<MailboxId>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let mut cached_mailboxes = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match inner.mailboxes.get(id) {
                Some(data) => cached_mailboxes.push(data.clone()),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_mailboxes,
            missing,
            state: inner.mailboxes_get_state.clone(),
        })
    }

    async fn get_all_mailboxes(
        &self,
    ) -> Result<cache::GetOneResult<Vec<MailboxData>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let mailboxes = inner.mailboxes.values().cloned().collect();

        Ok(cache::GetOneResult {
            value: mailboxes,
            state: inner.mailboxes_get_state.clone(),
        })
    }

    async fn upsert_mailboxes(
        &self,
        mailboxes: Vec<MailboxData>,
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for mailbox in mailboxes {
            let id = mailbox.id.clone();
            inner.mailboxes.insert(id, mailbox);
        }

        inner.mailboxes_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_mailboxes(
        &self,
        ids: &[MailboxId],
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for id in ids {
            inner.mailboxes.remove(id);
            inner.root_mails.remove(id);
        }

        inner.mailboxes_get_state = Some(new_state);
        Ok(())
    }

    async fn get_mailbox_children(
        &self,
        parent: &ParentMailboxId,
    ) -> Result<cache::GetOneResult<Option<Vec<MailboxData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let children = inner.mailboxes_get_state.is_some().then_some(
            inner
                .mailboxes
                .values()
                .filter(|mailbox| &mailbox.parent_id == parent)
                .cloned()
                .collect(),
        );

        Ok(cache::GetOneResult {
            value: children,
            state: inner.mailboxes_get_state.clone(),
        })
    }
}
