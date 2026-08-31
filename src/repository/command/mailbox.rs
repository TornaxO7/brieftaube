use super::Command;
use crate::{
    datasource::{Cache, Remote},
    repository::{self, Repository},
    types::{MailboxData, ParentMailboxId},
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum MailboxCommand<C, R>
where
    C: Cache,
    R: Remote,
{
    /// Get the child mailboxes of the given parent mailbox.
    GetChildren {
        id: ParentMailboxId,
        tx: oneshot::Sender<Result<Vec<MailboxData>, repository::Error<C, R>>>,
    },
}

impl<C, R> From<MailboxCommand<C, R>> for Command<C, R>
where
    C: Cache,
    R: Remote,
{
    fn from(cmd: MailboxCommand<C, R>) -> Self {
        Self::Mailbox(cmd)
    }
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    async fn ensure_mailboxes_are_cached(&self) -> Result<(), repository::Error<C, R>> {
        let mailboxes_are_fetched = self.cache.get_mailbox_state().is_some();
        if mailboxes_are_fetched {
            return Ok(());
        }

        let result = self
            .remote
            .fetch_mailboxes_all()
            .await
            .map_err(repository::Error::Remote)?;

        self.cache
            .upsert_mailboxes(result.values, result.state)
            .await
            .map_err(repository::Error::Cache)?;

        Ok(())
    }

    pub async fn mailbox_children(
        &self,
        id: ParentMailboxId,
    ) -> Result<Vec<MailboxData>, repository::Error<C, R>> {
        self.ensure_mailboxes_are_cached().await?;

        let result = self
            .cache
            .get_mailbox_children(&id)
            .await
            .map_err(repository::Error::Cache)?;

        Ok(result.value)
    }
}
