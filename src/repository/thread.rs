use super::Repository;
use crate::{
    datasource::types::remote,
    types::{AccountId, MailDataCore, MailId, ThreadId},
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
        tx: oneshot::Sender<color_eyre::Result<Vec<MailDataCore>>>,
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
    ) -> color_eyre::Result<Vec<MailDataCore>> {
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
                let opt_thread_mails = self
                    .caches
                    .get(&account_id)
                    .unwrap()
                    .read()
                    .await
                    .get_mails_core(&thread_mail_ids)
                    .await?;

                if opt_thread_mails.missing.is_empty() {
                    return Ok(opt_thread_mails.value);
                } else {
                    let result = self
                        .remote
                        .get_remote_account(account_id.clone())
                        .fetch_mails_core(&opt_thread_mails.missing)
                        .await?;

                    let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                    self.ensure_email_changes(&account_id, &result.state, &mut cache_lock)
                        .await?;

                    cache_lock
                        .upsert_mails_core(result.values.into_iter().collect())
                        .await?;

                    let thread_mail_cores_result =
                        cache_lock.get_mails_core(&thread_mail_ids).await?;

                    debug_assert!(thread_mail_cores_result.missing.is_empty());

                    return Ok(thread_mail_cores_result.value);
                }
            }
            None => {
                let remote::GetOneResult {
                    value:
                        remote::GetOneResult {
                            value: thread_mails,
                            state: get_mail_state,
                        },
                    state: thread_get_state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_thread(&id)
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &get_mail_state, &mut cache_lock)
                    .await?;

                self.ensure_thread_changes(&account_id, &thread_get_state, &mut cache_lock)
                    .await?;

                let thread_mail_ids: Vec<MailId> =
                    thread_mails.iter().map(|data| data.id.clone()).collect();

                cache_lock.upsert_mails_core(thread_mails.clone()).await?;
                cache_lock.upsert_thread(id, thread_mail_ids).await?;

                Ok(thread_mails)
            }
        }
    }
}
