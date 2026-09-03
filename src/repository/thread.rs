use super::{Error, Repository};
use crate::{
    datasource::{Cache, Remote, types::remote},
    types::{MailDataCore, MailId, ThreadId},
};
use std::sync::Mutex;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    GetThread {
        id: ThreadId,
        tx: oneshot::Sender<Result<Vec<MailDataCore>, Error<C, R>>>,
    },
}

impl<C, R> From<Command<C, R>> for super::Command<C, R>
where
    C: Cache,
    R: Remote,
{
    fn from(cmd: Command<C, R>) -> Self {
        Self::Thread(cmd)
    }
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub async fn get_thread(&self, id: ThreadId) -> Result<Vec<MailDataCore>, Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());

        let _enter_function = ENTER.lock().unwrap();
        let opt_thread_mail_ids = self
            .cache
            .read()
            .await
            .get_thread(&id)
            .await
            .map_err(Error::Cache)?;

        match opt_thread_mail_ids {
            Some(thread_mail_ids) => {
                let opt_thread_mails = self
                    .cache
                    .read()
                    .await
                    .get_mails_core(&thread_mail_ids)
                    .await
                    .map_err(Error::Cache)?;

                if opt_thread_mails.missing.is_empty() {
                    let thread_mails = thread_mail_ids
                        .into_iter()
                        .map(|id| opt_thread_mails.value.get(&id).cloned().unwrap())
                        .collect();
                    return Ok(thread_mails);
                } else {
                    let result = self
                        .remote
                        .fetch_mails_core(opt_thread_mails.missing)
                        .await
                        .map_err(Error::Remote)?;

                    let mut cache_lock = self.cache.write().await;
                    if let Some(current_state) = cache_lock.get_mail_state().await {
                        if *current_state != result.state {
                            self.apply_email_get_changes(&mut cache_lock).await?;
                        }
                    }

                    cache_lock
                        .upsert_mails_core(result.values)
                        .await
                        .map_err(Error::Cache)?;

                    let thread_mail_cores_result = cache_lock
                        .get_mails_core(&thread_mail_ids)
                        .await
                        .map_err(Error::Cache)?;
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
                } = self.remote.fetch_thread(&id).await.map_err(Error::Remote)?;

                let mut cache_lock = self.cache.write().await;

                let opt_current_email_get_state = cache_lock.get_mail_state().await;
                if opt_current_email_get_state
                    .is_some_and(|current_state| current_state != &get_mail_state)
                {
                    self.apply_email_get_changes(&mut cache_lock).await?;
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

                cache_lock
                    .upsert_mails_core(thread_mails)
                    .await
                    .map_err(Error::Cache)?;

                cache_lock
                    .upsert_thread(&id, thread_mail_ids)
                    .await
                    .map_err(Error::Cache)?;

                cache_lock
                    .set_mail_state(get_mail_state)
                    .await
                    .map_err(Error::Cache)?;

                cache_lock
                    .set_thread_state(thread_get_state)
                    .await
                    .map_err(Error::Cache)?;

                Ok(thread_mail_datas)
            }
        }
    }
}
