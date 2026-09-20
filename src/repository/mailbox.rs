use crate::{
    datasource::types::remote,
    repository::Repository,
    types::{AccountId, MailboxData, MailboxId, ParentMailboxId},
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
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
    async fn ensure_mailboxes_are_cached(&self, account_id: AccountId) -> color_eyre::Result<()> {
        let _enter = self.mailbox_locks.ensure_mailboxes_are_cached.lock().await;

        let mailboxes_are_fetched = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mailbox_state()
            .await
            .is_some();

        if mailboxes_are_fetched {
            return Ok(());
        }

        let remote::GetOneResult {
            value: mailboxes,
            state,
        } = self
            .remote
            .get_remote_account(account_id.clone())
            .fetch_mailboxes_all()
            .await?;

        self.caches
            .get(&account_id)
            .unwrap()
            .write()
            .await
            .upsert_mailboxes(mailboxes, state)
            .await?;

        Ok(())
    }

    pub async fn get_mailbox(
        &self,
        account_id: AccountId,
        id: MailboxId,
    ) -> color_eyre::Result<MailboxData> {
        self.ensure_mailboxes_are_cached(account_id.clone()).await?;

        let mailbox_data = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mailbox(&id)
            .await?
            .expect("Mailbox was fetched");

        Ok(mailbox_data)
    }

    pub async fn get_mailbox_children(
        &self,
        account_id: AccountId,
        id: ParentMailboxId,
    ) -> color_eyre::Result<Vec<MailboxData>> {
        self.ensure_mailboxes_are_cached(account_id.clone()).await?;

        let children = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mailbox_children(&id)
            .await?
            .expect("All mailboxes have been cached");

        Ok(children)
    }
}
