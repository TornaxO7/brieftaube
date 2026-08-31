use super::{Error, Repository};
use crate::{
    datasource::{
        Cache, Remote,
        types::{cache, remote},
    },
    types::{MailData, MailId, ThreadId},
};
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
        let cache::GetOneResult {
            value: opt_thread_mails,
            ..
        } = self.cache.get_thread(&id).await.map_err(Error::Cache)?;

        match opt_thread_mails {
            Some(thread_mails) => Ok(thread_mails),
            None => {
                let remote::GetOneResult {
                    value: thread_mails_result,
                    state: thread_get_state,
                } = self.remote.fetch_thread(&id).await.map_err(Error::Remote)?;
                let remote::GetOneResult {
                    value: thread_mails,
                    state: get_mail_state,
                } = thread_mails_result;

                self.cache
                    .upsert_thread(&id, &thread_mails, get_mail_state, thread_get_state)
                    .await
                    .map_err(Error::Cache)?;

                Ok(thread_mails)
            }
        }
    }
}
