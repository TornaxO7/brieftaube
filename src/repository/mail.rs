use crate::{
    datasource::{
        Cache, Remote,
        types::{QueryWindow, cache, remote},
    },
    repository,
    types::{MailData, MailboxId},
};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    QueryRootMails {
        mailbox: MailboxId,
        start: i32,
        limit: u32,
        tx: oneshot::Sender<Result<Vec<MailData>, repository::Error<C, R>>>,
    },
}

impl<C, R> From<Command<C, R>> for super::Command<C, R>
where
    C: Cache,
    R: Remote,
{
    fn from(cmd: Command<C, R>) -> Self {
        Self::Mail(cmd)
    }
}

impl<C, R> repository::Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub async fn query_root_mails(
        &self,
        id: MailboxId,
        start: i32,
        limit: u32,
    ) -> Result<Vec<MailData>, repository::Error<C, R>> {
        let mailbox = self.get_mailbox(id.clone()).await?;
        let amount_threads = mailbox.total_threads;

        let window = {
            let s = if start < 0 {
                // according to spec (see `position` from `/query` in `core`)
                (amount_threads as i32 + start).max(0) as u32
            } else {
                start as u32
            };

            QueryWindow {
                start: s,
                limit: limit as usize,
            }
        };

        let result = self
            .cache
            .query_root_mails(&id, window.clone())
            .await
            .map_err(repository::Error::Cache)?;

        let mail_ids = if result.missing.is_empty() && result.is_initialised() {
            debug_assert_eq!(result.values.len(), 1, "Full window should be loaded");
            result.values.into_iter().next().unwrap().ids
        } else {
            // TODO: Only fetch the missing windows => Redces potentially big requests:
            // 1. Create one request-batch for each window
            // 2. Add each of them into the cache
            // 3. Query them all again
            let remote::QueryResponse { ids, state } = self
                .remote
                .fetch_root_mails(&id, &window)
                .await
                .map_err(repository::Error::Remote)?;

            self.cache
                .upsert_root_mails(&id, window.start as usize, ids.clone(), state)
                .await
                .map_err(repository::Error::Cache)?;

            ids
        };

        let cache::GetBatchResult {
            value: cache_mails,
            missing: missing_cache_mails,
            ..
        } = self
            .cache
            .get_mails(&mail_ids)
            .await
            .map_err(repository::Error::Cache)?;

        let cache_mails = if missing_cache_mails.is_empty() {
            cache_mails
        } else {
            // fetch the missing mails
            {
                let result = self
                    .remote
                    .fetch_mails(&missing_cache_mails)
                    .await
                    .map_err(repository::Error::Remote)?;

                if !result.not_found.is_empty() {
                    todo!("Eeh... that's... kinda sus. Don't know (yet)");
                }

                self.cache
                    .upsert_mails(result.values.clone(), result.state)
                    .await
                    .map_err(repository::Error::Cache)?;
            }

            let result = self
                .cache
                .get_mails(&mail_ids)
                .await
                .map_err(repository::Error::Cache)?;

            debug_assert!(
                result.missing.is_empty(),
                "Mails should've been added just now o.O"
            );

            result.value
        };

        Ok(cache_mails)
    }
}
