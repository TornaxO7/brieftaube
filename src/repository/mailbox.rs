use crate::{
    datasource::{Cache, Remote, types::remote},
    repository::{self, Repository},
    types::{MailboxData, MailboxId, ParentMailboxId},
};
use std::sync::Mutex;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command<C, R>
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

impl<C, R> From<Command<C, R>> for super::Command<C, R>
where
    C: Cache,
    R: Remote,
{
    fn from(cmd: Command<C, R>) -> Self {
        Self::Mailbox(cmd)
    }
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    async fn ensure_mailboxes_are_cached(&self) -> Result<(), repository::Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());
        let _enter_function = ENTER.lock().unwrap();

        let mailboxes_are_fetched = self.cache.read().await.get_mailbox_state().await.is_some();
        if mailboxes_are_fetched {
            return Ok(());
        }

        let remote::GetOneResult {
            value: mailboxes,
            state,
        } = self
            .remote
            .fetch_mailboxes_all()
            .await
            .map_err(repository::Error::Remote)?;

        self.cache
            .write()
            .await
            .upsert_mailboxes(mailboxes, state)
            .await
            .map_err(repository::Error::Cache)?;

        Ok(())
    }

    pub async fn get_mailbox(&self, id: MailboxId) -> Result<MailboxData, repository::Error<C, R>> {
        self.ensure_mailboxes_are_cached().await?;

        let mailbox_data = self
            .cache
            .read()
            .await
            .get_mailbox(&id)
            .await
            .map_err(repository::Error::Cache)?
            .expect("Mailbox was fetched");

        Ok(mailbox_data)
    }

    pub async fn get_mailbox_children(
        &self,
        id: ParentMailboxId,
    ) -> Result<Vec<MailboxData>, repository::Error<C, R>> {
        self.ensure_mailboxes_are_cached().await?;

        let children = self
            .cache
            .read()
            .await
            .get_mailbox_children(&id)
            .await
            .map_err(repository::Error::Cache)?
            .expect("All mailboxes have been cached");

        Ok(children)
    }
}
