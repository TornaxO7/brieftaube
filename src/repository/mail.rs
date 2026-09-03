use crate::{
    datasource::{
        Cache, Remote,
        types::{QueryWindow, remote},
    },
    repository::{Error, Repository},
    types::{MailDataCore, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
};
use std::sync::Mutex;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    GetTextBody {
        id: MailId,
        tx: oneshot::Sender<Result<MailDataTextBody, Error<C, R>>>,
    },
    GetHtmlBody {
        id: MailId,
        tx: oneshot::Sender<Result<MailDataHtmlBody, Error<C, R>>>,
    },
    QueryRootMails {
        mailbox: MailboxId,
        start: i32,
        limit: u32,
        tx: oneshot::Sender<Result<Vec<MailDataCore>, Error<C, R>>>,
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

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub async fn get_mail_text_body(&self, id: MailId) -> Result<MailDataTextBody, Error<C, R>> {
        // don't let another task read from the cache while another task is currently requesting the data
        static ENTER: Mutex<()> = Mutex::new(());
        let _enter_function = ENTER.lock().unwrap();

        let opt_text_body = self
            .cache
            .read()
            .await
            .get_mail_text_body(&id)
            .await
            .map_err(Error::Cache)?;

        match opt_text_body {
            Some(text_body) => Ok(text_body),
            None => {
                let remote::GetOneResult {
                    value: text_body,
                    state,
                } = self
                    .remote
                    .fetch_mail_text_body(&id)
                    .await
                    .map_err(Error::Remote)?;

                let mut cache_lock = self.cache.write().await;
                let opt_current_state = cache_lock.get_mail_state().await;
                if opt_current_state.is_some_and(|current_state| *current_state != state) {
                    self.apply_email_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&state));

                cache_lock
                    .upsert_mail_text_body(&id, text_body.clone())
                    .await
                    .map_err(Error::Cache)?;

                Ok(text_body)
            }
        }
    }

    pub async fn get_mail_html_body(&self, id: MailId) -> Result<MailDataHtmlBody, Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());
        let _enter_function = ENTER.lock().unwrap();

        let opt_html_body = self
            .cache
            .read()
            .await
            .get_mail_html_body(&id)
            .await
            .map_err(Error::Cache)?;

        match opt_html_body {
            Some(html_body) => Ok(html_body),
            None => {
                let remote::GetOneResult {
                    value: html_body,
                    state,
                } = self
                    .remote
                    .fetch_mail_html_body(&id)
                    .await
                    .map_err(Error::Remote)?;

                let mut cache_lock = self.cache.write().await;
                let opt_current_state = cache_lock.get_mail_state().await;
                if opt_current_state.is_some_and(|current_state| *current_state != state) {
                    self.apply_email_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&state));

                cache_lock
                    .upsert_mail_html_body(&id, html_body.clone())
                    .await
                    .map_err(Error::Cache)?;

                Ok(html_body)
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        id: MailboxId,
        start: i32,
        limit: u32,
    ) -> Result<Vec<MailDataCore>, Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());

        let mailbox = self.get_mailbox(id.clone()).await?;
        let amount_threads = mailbox.total_threads;

        let window = {
            let normalized_start = if start < 0 {
                // according to spec (see `position` from `/query` in `core`)
                (amount_threads as i32 + start).max(0) as u32
            } else {
                start as u32
            };

            QueryWindow {
                start: normalized_start,
                limit: limit as usize,
            }
        };

        let _enter_function = ENTER.lock().unwrap();

        let opt_root_mail_ids = self
            .cache
            .read()
            .await
            .query_root_mails(&id, window.clone())
            .await
            .map_err(Error::Cache)?;

        if let Some(root_mails) = opt_root_mail_ids
            && root_mails.missing.is_empty()
        {
            debug_assert_eq!(root_mails.values.len(), 1, "Full window was loaded");
            let root_mails = root_mails.values.into_iter().next().unwrap().values;

            let opt_root_mails = self
                .cache
                .read()
                .await
                .get_mails_core(&root_mails)
                .await
                .map_err(Error::Cache)?;

            if opt_root_mails.missing.is_empty() {
                let root_mails_core = root_mails
                    .into_iter()
                    .map(|id| opt_root_mails.value.get(&id).cloned().unwrap())
                    .collect();
                return Ok(root_mails_core);
            } else {
                let missing_mails_core = self
                    .remote
                    .fetch_mails_core(opt_root_mails.missing)
                    .await
                    .map_err(Error::Remote)?;

                let mut cache_lock = self.cache.write().await;
                if let Some(current_email_get_state) = cache_lock.get_mail_state().await {
                    if *current_email_get_state != missing_mails_core.state {
                        self.apply_email_get_changes(&mut cache_lock).await?;
                    }
                }

                cache_lock
                    .upsert_mails_core(missing_mails_core.values)
                    .await
                    .map_err(Error::Cache)?;

                let result = cache_lock
                    .get_mails_core(&root_mails)
                    .await
                    .map_err(Error::Cache)?;

                debug_assert!(result.missing.is_empty());

                let root_mails_core = root_mails
                    .into_iter()
                    .map(|id| result.value.get(&id).cloned().unwrap())
                    .collect();

                return Ok(root_mails_core);
            }
        }

        // PERFORMANCE: Instead of a full fetch of the window, maybe we could just fetch the missing sections

        let remote::QueryResponse {
            value:
                remote::GetOneResult {
                    value: root_mails,
                    state: email_get_state,
                },
            state: root_mails_query_state,
        } = self
            .remote
            .fetch_root_mails(&id, &window)
            .await
            .map_err(Error::Remote)?;

        let mut cache_lock = self.cache.write().await;

        if let Some(current_email_get_state) = cache_lock.get_mail_state().await {
            if *current_email_get_state != email_get_state {
                self.apply_email_get_changes(&mut cache_lock).await?;
            }
        }

        if let Some(current_root_mail_query_state) = cache_lock.get_root_mails_state(&id).await {
            if *current_root_mail_query_state != root_mails_query_state {
                self.apply_root_mail_query_changes(&id, &mut cache_lock)
                    .await?;
            }
        }

        debug_assert_eq!(cache_lock.get_mail_state().await, Some(&email_get_state));
        debug_assert_eq!(
            cache_lock.get_root_mails_state(&id).await,
            Some(&root_mails_query_state)
        );

        let cache_root_mails: Vec<(MailId, usize)> = root_mails
            .iter()
            .enumerate()
            .map(|(idx, (id, _root_mail_core))| {
                let position = window.start as usize + idx;
                (id.clone(), position)
            })
            .collect();

        cache_lock
            .insert_root_mails(&id, cache_root_mails)
            .await
            .map_err(Error::Cache)?;

        cache_lock
            .upsert_mails_core(root_mails.clone())
            .await
            .map_err(Error::Cache)?;

        cache_lock
            .set_root_mails_state(&id, root_mails_query_state)
            .await
            .map_err(Error::Cache)?;

        let root_mails_core = root_mails.into_iter().map(|(_id, data)| data).collect();

        Ok(root_mails_core)
    }
}
