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
    ) -> Result<cache::GetResult<Vec<Option<MailboxData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let mailboxes: Vec<Option<MailboxData>> = ids
            .iter()
            .map(|id| inner.mailboxes.get(id).cloned())
            .collect();

        Ok(cache::GetResult {
            value: mailboxes,
            state: inner.mailboxes_get_state.clone(),
        })
    }

    async fn get_all_mailboxes(&self) -> Result<cache::GetResult<Vec<MailboxData>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let mailboxes = inner.mailboxes.values().cloned().collect();

        Ok(cache::GetResult {
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
    ) -> Result<cache::GetResult<Vec<MailboxData>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let children = inner
            .mailboxes
            .values()
            .filter(|mailbox| &mailbox.parent_id == parent)
            .cloned()
            .collect();

        Ok(cache::GetResult {
            value: children,
            state: inner.mailboxes_get_state.clone(),
        })
    }
}
