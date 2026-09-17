use crate::{
    datasource::types::remote,
    repository::Repository,
    types::{MailboxData, MailboxId, ParentMailboxId},
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub enum Command {
    /// Get the child mailboxes of the given parent mailbox.
    GetChildren {
        id: ParentMailboxId,
        tx: oneshot::Sender<color_eyre::Result<Vec<MailboxData>>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::Mailbox(cmd)
    }
}

#[derive(Default)]
pub struct Locks {
    ensure_mailboxes_are_cached: Mutex<()>,
}

impl Repository {
    async fn ensure_mailboxes_are_cached(&self) -> color_eyre::Result<()> {
        let _enter = self.mailbox_locks.ensure_mailboxes_are_cached.lock().await;

        let mailboxes_are_fetched = self.cache.read().await.get_mailbox_state().await.is_some();
        if mailboxes_are_fetched {
            return Ok(());
        }

        let remote::GetOneResult {
            value: mailboxes,
            state,
        } = self.remote.fetch_mailboxes_all().await?;

        self.cache
            .write()
            .await
            .upsert_mailboxes(mailboxes, state)
            .await?;

        Ok(())
    }

    pub async fn get_mailbox(&self, id: MailboxId) -> color_eyre::Result<MailboxData> {
        self.ensure_mailboxes_are_cached().await?;

        let mailbox_data = self
            .cache
            .read()
            .await
            .get_mailbox(&id)
            .await?
            .expect("Mailbox was fetched");

        Ok(mailbox_data)
    }

    pub async fn get_mailbox_children(
        &self,
        id: ParentMailboxId,
    ) -> color_eyre::Result<Vec<MailboxData>> {
        self.ensure_mailboxes_are_cached().await?;

        let children = self
            .cache
            .read()
            .await
            .get_mailbox_children(&id)
            .await?
            .expect("All mailboxes have been cached");

        Ok(children)
    }
}
