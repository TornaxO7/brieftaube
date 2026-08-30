use crate::{
    datasources::{
        MailboxCache, MailboxDataSource,
        hashmap::HashMapDataSource,
        types::{LocalGetResult, GetState},
    },
    types::{MailboxData, MailboxId},
};

impl MailboxDataSource for HashMapDataSource {
    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<LocalGetResult<Vec<Option<MailboxData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();

        let mailboxes: Vec<Option<MailboxData>> = ids
            .iter()
            .map(|id| inner.mailboxes.get(id).cloned())
            .collect();

        Ok(LocalGetResult {
            value: mailboxes,
            state: inner.mailboxes_get_state.clone(),
        })
    }

    async fn get_all_mailboxes(&self) -> Result<LocalGetResult<Vec<MailboxData>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let mailboxes = inner.mailboxes.values().cloned().collect();

        Ok(LocalGetResult {
            value: mailboxes,
            state: inner.mailboxes_get_state.clone(),
        })
    }
}

impl MailboxCache for HashMapDataSource {
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
}
