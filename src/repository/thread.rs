use super::{Error, Repository};
use crate::{
    datasource::{Cache, Remote, types::remote},
    types::{MailData, ThreadId},
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
        tx: oneshot::Sender<Result<Vec<MailData>, Error<C, R>>>,
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
    pub async fn get_thread(&self, id: ThreadId) -> Result<Vec<MailData>, Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());

        let _enter_function = ENTER.lock().unwrap();
        let opt_thread_mails = self
            .cache
            .read()
            .await
            .get_thread(&id)
            .await
            .map_err(Error::Cache)?;

        match opt_thread_mails {
            Some(thread_mails) => Ok(thread_mails),
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

                cache_lock
                    .upsert_thread(&id, &thread_mails, get_mail_state, thread_get_state)
                    .await
                    .map_err(Error::Cache)?;

                Ok(thread_mails)
            }
        }
    }
}
