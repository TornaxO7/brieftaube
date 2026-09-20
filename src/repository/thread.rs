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
                    let thread_mails = thread_mail_ids
                        .into_iter()
                        .map(|id| opt_thread_mails.value.get(&id).cloned().unwrap())
                        .collect();
                    return Ok(thread_mails);
                } else {
                    let result = self
                        .remote
                        .get_remote_account(account_id.clone())
                        .fetch_mails_core(&opt_thread_mails.missing)
                        .await?;

                    let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;
                    if let Some(current_state) = cache_lock.get_mail_state().await {
                        if *current_state != result.state {
                            self.apply_email_get_changes(&account_id, &mut cache_lock)
                                .await?;
                        }
                    }

                    cache_lock
                        .upsert_mails_core(result.values.into_iter().collect())
                        .await?;

                    let thread_mail_cores_result =
                        cache_lock.get_mails_core(&thread_mail_ids).await?;
                    debug_assert!(thread_mail_cores_result.missing.is_empty());

                    let thread_mail_cores = thread_mail_ids
                        .into_iter()
                        .map(|id| thread_mail_cores_result.value.get(&id).cloned().unwrap())
                        .collect();

                    return Ok(thread_mail_cores);
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

                let opt_current_email_get_state = cache_lock.get_mail_state().await;
                if opt_current_email_get_state
                    .is_some_and(|current_state| current_state != &get_mail_state)
                {
                    self.apply_email_get_changes(&account_id, &mut cache_lock)
                        .await?;
                }

                let opt_current_thread_get_state = cache_lock.get_thread_state().await;
                if opt_current_thread_get_state
                    .is_some_and(|current_state| current_state != &thread_get_state)
                {
                    self.apply_thread_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&get_mail_state));
                debug_assert_eq!(cache_lock.get_thread_state().await, Some(&thread_get_state));

                let thread_mail_ids: Vec<MailId> =
                    thread_mails.iter().map(|(id, _data)| id.clone()).collect();

                let thread_mail_datas: Vec<MailDataCore> = thread_mails
                    .iter()
                    .map(|(_id, data)| data.clone())
                    .collect();

                cache_lock.upsert_mails_core(thread_mails).await?;
                cache_lock.upsert_thread(id, thread_mail_ids).await?;
                cache_lock.set_mail_state(get_mail_state).await?;
                cache_lock.set_thread_state(thread_get_state).await?;

                Ok(thread_mail_datas)
            }
        }
    }
}
