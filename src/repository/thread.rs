use super::Repository;
use crate::{
    datasource::types::{cache, remote},
    types::{AccountId, CachedMail, MailId, MailProperty, ThreadId},
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    GetThread {
        id: ThreadId,
        properties: Vec<MailProperty>,
        tx: oneshot::Sender<color_eyre::Result<Vec<CachedMail>>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::Thread(cmd)
    }
}

#[derive(Default)]
pub struct Locks {
    get_thread: Mutex<()>,
}

impl Repository {
    pub async fn get_thread(
        &self,
        account_id: AccountId,
        id: ThreadId,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<Vec<CachedMail>> {
        let _enter = self.thread_locks.get_thread.lock().await;

        let opt_thread_mail_ids = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_thread(&id)
            .await?;

        match opt_thread_mail_ids {
            Some(thread_mail_ids) => {
                let cache::GetBatchResult {
                    value: mut cached_mails,
                    missing: missing_thread_mails,
                } = self
                    .caches
                    .get(&account_id)
                    .unwrap()
                    .read()
                    .await
                    .get_mails(&thread_mail_ids)
                    .await?;

                debug_assert!(
                    missing_thread_mails.is_empty(),
                    "All mails from a thread must be there."
                );

                if cached_mails
                    .iter()
                    .all(|cached_mail| cached_mail.has_properties(&properties))
                {
                    return Ok(cached_mails);
                }

                let fetched_mails = {
                    let cached_mail_ids: Vec<MailId> = cached_mails
                        .iter()
                        .map(|cached_mail| cached_mail.id.clone())
                        .collect();

                    self.remote
                        .get_remote_account(account_id.clone())
                        .fetch_mails(&cached_mail_ids, properties)
                        .await?
                };

                for fetched_mail in fetched_mails.values {
                    let cached_mail = cached_mails
                        .iter_mut()
                        .find(|cached_mail| cached_mail.id == fetched_mail.id)
                        .unwrap();

                    cached_mail.merge(fetched_mail);
                }

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &fetched_mails.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(cached_mails.clone()).await?;
                Ok(cached_mails)
            }
            None => {
                let remote::GetOneResult {
                    value:
                        remote::GetOneResult {
                            value: fetched_thread_mails,
                            state: get_mail_state,
                        },
                    state: thread_get_state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_thread_with_mails(&id, properties.clone())
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &get_mail_state, &mut cache_lock)
                    .await?;

                self.ensure_thread_changes(&account_id, &thread_get_state, &mut cache_lock)
                    .await?;

                let thread_mail_ids: Vec<MailId> = fetched_thread_mails
                    .iter()
                    .map(|data| data.id.clone())
                    .collect();

                cache_lock
                    .upsert_mails(fetched_thread_mails.clone())
                    .await?;
                cache_lock.upsert_thread(id, thread_mail_ids).await?;

                Ok(fetched_thread_mails)
            }
        }
    }
}
