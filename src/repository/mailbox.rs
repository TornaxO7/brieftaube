use crate::{
    datasource::types::{MailboxPatch, QueryWindow, remote},
    repository::Repository,
    types::{AccountId, CachedMailbox, InitMailboxData, MailProperty, MailboxId, ParentMailboxId},
};
use std::{collections::HashMap, sync::OnceLock};
use tokio::sync::{Mutex, oneshot};
use tracing::error;

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    Init {
        id: ParentMailboxId,
        mail_window: QueryWindow,
        mail_properties: Vec<MailProperty>,

        tx: oneshot::Sender<color_eyre::Result<InitMailboxData>>,
    },
    Patch {
        patches: Vec<MailboxPatch>,

        tx: oneshot::Sender<color_eyre::Result<()>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::Mailbox(cmd)
    }
}

#[derive(Default)]
pub struct Locks {
    get_mailbox: Mutex<()>,
}

impl Repository {
    async fn ensure_mailboxes_are_cached(&self, account_id: AccountId) -> color_eyre::Result<()> {
        let _enter = self.mailbox_locks.get_mailbox.lock().await;

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
            .fetch_mailboxes_all(CachedMailbox::PROPERTIES.to_vec())
            .await?;

        let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;
        cache_lock.upsert_mailboxes(mailboxes).await?;
        cache_lock.set_mailbox_state(state).await?;

        Ok(())
    }

    pub async fn get_all_mailboxes(
        &self,
        account_id: AccountId,
    ) -> color_eyre::Result<Vec<CachedMailbox>> {
        self.ensure_mailboxes_are_cached(account_id.clone()).await?;

        let mailboxes = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_all_mailboxes()
            .await?
            .expect("All mailboxes fetched");

        Ok(mailboxes)
    }

    pub async fn get_mailbox(
        &self,
        account_id: AccountId,
        id: MailboxId,
    ) -> color_eyre::Result<CachedMailbox> {
        self.ensure_mailboxes_are_cached(account_id.clone()).await?;

        let mailbox_data = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mailbox(id.clone())
            .await?
            .expect(&format!("Mailbox '{:?}' was fetched", id));

        Ok(mailbox_data)
    }

    pub async fn get_mailbox_children(
        &self,
        account_id: AccountId,
        id: ParentMailboxId,
    ) -> color_eyre::Result<Vec<CachedMailbox>> {
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

    pub async fn get_init_mailbox_data(
        &self,
        account_id: AccountId,
        id: ParentMailboxId,
        mail_window: QueryWindow,
        mail_properties: Vec<MailProperty>,
    ) -> color_eyre::Result<InitMailboxData> {
        let child_mailboxes = self
            .get_mailbox_children(account_id.clone(), id.clone())
            .await?;

        match id {
            Some(id) => {
                let data = self.get_mailbox(account_id.clone(), id.clone()).await?;

                let first_mails = self
                    .query_root_mails(account_id, id, mail_window, mail_properties)
                    .await?;

                Ok(InitMailboxData {
                    total_threads: data.total_threads.get().copied().unwrap(),
                    child_mailboxes,
                    first_mails,
                })
            }
            None => Ok(InitMailboxData {
                total_threads: 0,
                child_mailboxes,
                first_mails: vec![],
            }),
        }
    }

    pub async fn patch_mailboxes(
        &self,
        account_id: AccountId,
        patches: Vec<MailboxPatch>,
    ) -> color_eyre::Result<()> {
        let _enter = self.mailbox_locks.get_mailbox.lock().await;

        let ids: Vec<MailboxId> = patches.iter().map(|mailbox| mailbox.id.clone()).collect();

        match self
            .remote
            .get_remote_account(account_id.clone())
            .patch_mailboxes(patches.clone())
            .await
        {
            Ok(result) => {
                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mailbox_changes(&account_id, &result.new_state, &mut cache_lock)
                    .await?;

                let mut impacted_mailboxes = cache_lock
                    .get_mailboxes(&ids)
                    .await?
                    .value
                    .into_iter()
                    .map(|mailbox| (mailbox.id.clone(), mailbox))
                    .collect::<HashMap<MailboxId, CachedMailbox>>();

                for patch in patches {
                    let mailbox = impacted_mailboxes.get_mut(&patch.id).unwrap();

                    if let Some(new_name) = patch.new_name {
                        mailbox.name = OnceLock::from(new_name);
                    }

                    if let Some(new_parent_id) = patch.new_parent_id {
                        mailbox.parent_id = OnceLock::from(new_parent_id);
                    }

                    if let Some(new_role) = patch.new_role {
                        mailbox.role = OnceLock::from(new_role);
                    }

                    if let Some(new_sort_order) = patch.new_sort_order {
                        mailbox.sort_order = OnceLock::from(new_sort_order);
                    }
                }

                cache_lock
                    .upsert_mailboxes(impacted_mailboxes.into_values().collect())
                    .await?;
                Ok(())
            }
            Err(err) => {
                error!("Couldn't udpate mailboxes: {}", err);
                Err(err)
            }
        }
    }
}
