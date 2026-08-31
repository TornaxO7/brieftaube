use crate::{
    datasource::{
        MailboxCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailboxData, MailboxId, ParentMailboxId},
};

impl MailboxCache for HashMapDataSource {
    async fn get_mailbox_state(&self) -> Option<&GetState> {
        self.mailboxes_get_state.as_ref()
    }

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetBatchResult<Vec<MailboxData>, Vec<MailboxId>>, Self::Error> {
        let mut cached_mailboxes = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mailboxes.get(id) {
                Some(data) => cached_mailboxes.push(data.clone()),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_mailboxes,
            missing,
        })
    }

    async fn get_all_mailboxes(&self) -> Result<Option<Vec<MailboxData>>, Self::Error> {
        if self.mailboxes_get_state.is_some() {
            let mailboxes = self.mailboxes.values().cloned().collect();
            Ok(Some(mailboxes))
        } else {
            Ok(None)
        }
    }

    async fn upsert_mailboxes(
        &mut self,
        mailboxes: Vec<MailboxData>,
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        for mailbox in mailboxes {
            let id = mailbox.id.clone();
            self.mailboxes.insert(id, mailbox);
        }

        self.mailboxes_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_mailboxes(
        &mut self,
        ids: &[MailboxId],
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        for id in ids {
            self.mailboxes.remove(id);
            self.root_mails.remove(id);
        }

        self.mailboxes_get_state = Some(new_state);
        Ok(())
    }

    async fn get_mailbox_children(
        &self,
        parent: &ParentMailboxId,
    ) -> Result<Option<Vec<MailboxData>>, Self::Error> {
        let children = self.mailboxes_get_state.is_some().then_some(
            self.mailboxes
                .values()
                .filter(|mailbox| &mailbox.parent_id == parent)
                .cloned()
                .collect(),
        );

        Ok(children)
    }
}
