use crate::{
    datasource::{
        Cache, Remote,
        types::{QueryWindow, remote},
    },
    repository::{Error, Repository},
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
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
    GetAttachments {
        id: MailId,
        tx: oneshot::Sender<Result<Vec<MailDataAttachment>, Error<C, R>>>,
    },
    QueryRootMails {
        mailbox: MailboxId,
        start: i32,
        limit: u32,
        tx: oneshot::Sender<Result<Vec<MailData>, Error<C, R>>>,
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

    pub async fn get_mail_attachments(
        &self,
        id: MailId,
    ) -> Result<Vec<MailDataAttachment>, Error<C, R>> {
        static ENTER: Mutex<()> = Mutex::new(());
        let _enter_function = ENTER.lock().unwrap();

        let opt_mail_attachments = self
            .cache
            .read()
            .await
            .get_mail_attachments(&id)
            .await
            .map_err(Error::Cache)?;

        match opt_mail_attachments {
            Some(mail_attachments) => Ok(mail_attachments),
            None => {
                let remote::GetOneResult {
                    value: mail_attachments,
                    state,
                } = self
                    .remote
                    .fetch_mail_attachments(&id)
                    .await
                    .map_err(Error::Remote)?;

                let mut cache_lock = self.cache.write().await;
                let opt_current_state = cache_lock.get_mail_state().await.cloned();
                if opt_current_state.is_some_and(|current_state| current_state != state) {
                    self.apply_email_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&state));

                cache_lock
                    .upsert_mail_attachments(&id, mail_attachments.clone())
                    .await
                    .map_err(Error::Cache)?;

                Ok(mail_attachments)
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        id: MailboxId,
        start: i32,
        limit: u32,
    ) -> Result<Vec<MailData>, Error<C, R>> {
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

        let opt_root_mails = self
            .cache
            .read()
            .await
            .query_root_mails(&id, window.clone())
            .await
            .map_err(Error::Cache)?;

        if let Some(root_mails) = opt_root_mails
            && root_mails.missing.is_empty()
        {
            debug_assert_eq!(root_mails.values.len(), 1, "Full window was loaded");
            return Ok(root_mails.values.into_iter().next().unwrap().values);
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
                self.apply_root_mail_query_changes(&id, &window, &mut cache_lock)
                    .await?;
            }
        }

        debug_assert_eq!(cache_lock.get_mail_state().await, Some(&email_get_state));
        debug_assert_eq!(
            cache_lock.get_root_mails_state(&id).await,
            Some(&root_mails_query_state)
        );

        let cache_root_mails = root_mails
            .clone()
            .into_iter()
            .enumerate()
            .map(|(idx, root_mail)| {
                let position = window.start as usize + idx;
                (root_mail, position)
            })
            .collect();

        cache_lock
            .insert_root_mails(&id, cache_root_mails)
            .await
            .map_err(Error::Cache)?;

        Ok(root_mails)
    }
}
