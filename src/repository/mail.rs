use crate::{
    datasource::types::{QueryWindow, remote},
    repository::Repository,
    types::{MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailId, MailboxId},
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub enum CommandKind {
    GetCore {
        id: MailId,
        tx: oneshot::Sender<color_eyre::Result<MailDataCore>>,
    },
    GetPreview {
        id: MailId,
        tx: oneshot::Sender<color_eyre::Result<MailDataPreview>>,
    },
    GetTextBody {
        id: MailId,
        tx: oneshot::Sender<color_eyre::Result<MailDataTextBody>>,
    },
    GetHtmlBody {
        id: MailId,
        tx: oneshot::Sender<color_eyre::Result<MailDataHtmlBody>>,
    },
    QueryRootMails {
        mailbox: MailboxId,
        start: i32,
        limit: u32,
        tx: oneshot::Sender<color_eyre::Result<Vec<MailDataCore>>>,
    },
}

impl From<CommandKind> for super::CommandKind {
    fn from(cmd: CommandKind) -> Self {
        Self::Mail(cmd)
    }
}

#[derive(Default)]
pub struct Locks {
    get_mail_core: Mutex<()>,
    get_mail_preview: Mutex<()>,
    get_mail_text_body: Mutex<()>,
    get_mail_html_body: Mutex<()>,
    query_root_mails: Mutex<()>,
}

impl Repository {
    pub async fn get_mail_core(&self, id: MailId) -> color_eyre::Result<MailDataCore> {
        let _entry = self.mail_locks.get_mail_core.lock().await;

        match self.cache.read().await.get_mail_core(&id).await? {
            Some(data) => Ok(data),
            None => {
                let result = self.remote.fetch_mail_core(id.clone()).await?;

                let mut cache_lock = self.cache.write().await;
                if let Some(current_email_get_state) = cache_lock.get_mail_state().await {
                    if *current_email_get_state != result.state {
                        self.apply_email_get_changes(&mut cache_lock).await?;
                    }
                }

                cache_lock
                    .upsert_mails_core(vec![(id, result.value.clone())])
                    .await?;

                cache_lock.set_mail_state(result.state).await?;

                Ok(result.value)
            }
        }
    }

    pub async fn get_mail_preview(&self, id: MailId) -> color_eyre::Result<MailDataPreview> {
        let _enter = self.mail_locks.get_mail_preview.lock().await;

        match self.cache.read().await.get_mail_preview(&id).await? {
            Some(data) => Ok(data),
            None => {
                let result = self.remote.fetch_mail_preview(id.clone()).await?;

                let mut cache_lock = self.cache.write().await;
                if let Some(current_email_get_state) = cache_lock.get_mail_state().await {
                    if *current_email_get_state != result.state {
                        self.apply_email_get_changes(&mut cache_lock).await?;
                    }
                }

                cache_lock
                    .upsert_mails_preview(vec![(id, result.value.clone())])
                    .await?;

                cache_lock.set_mail_state(result.state).await?;

                Ok(result.value)
            }
        }
    }

    pub async fn get_mail_text_body(&self, id: MailId) -> color_eyre::Result<MailDataTextBody> {
        let _enter = self.mail_locks.get_mail_text_body.lock().await;

        let opt_text_body = self.cache.read().await.get_mail_text_body(&id).await?;

        match opt_text_body {
            Some(text_body) => Ok(text_body),
            None => {
                let remote::GetOneResult {
                    value: text_body,
                    state,
                } = self.remote.fetch_mail_text_body(&id).await?;

                let mut cache_lock = self.cache.write().await;
                let opt_current_state = cache_lock.get_mail_state().await;
                if opt_current_state.is_some_and(|current_state| *current_state != state) {
                    self.apply_email_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&state));

                cache_lock
                    .upsert_mail_text_body(&id, text_body.clone())
                    .await?;

                Ok(text_body)
            }
        }
    }

    pub async fn get_mail_html_body(&self, id: MailId) -> color_eyre::Result<MailDataHtmlBody> {
        let _enter = self.mail_locks.get_mail_html_body.lock().await;

        let opt_html_body = self.cache.read().await.get_mail_html_body(&id).await?;

        match opt_html_body {
            Some(html_body) => Ok(html_body),
            None => {
                let remote::GetOneResult {
                    value: html_body,
                    state,
                } = self.remote.fetch_mail_html_body(&id).await?;

                let mut cache_lock = self.cache.write().await;
                let opt_current_state = cache_lock.get_mail_state().await;
                if opt_current_state.is_some_and(|current_state| *current_state != state) {
                    self.apply_email_get_changes(&mut cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await, Some(&state));

                cache_lock
                    .upsert_mail_html_body(&id, html_body.clone())
                    .await?;

                Ok(html_body)
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        id: MailboxId,
        start: i32,
        limit: u32,
    ) -> color_eyre::Result<Vec<MailDataCore>> {
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

        let _enter = self.mail_locks.query_root_mails.lock().await;

        let opt_root_mail_ids = self
            .cache
            .read()
            .await
            .query_root_mails(&id, window.clone())
            .await?;

        if let Some(root_mails) = opt_root_mail_ids
            && root_mails.missing.is_empty()
        {
            debug_assert_eq!(root_mails.values.len(), 1, "Full window was loaded");
            let root_mails = root_mails.values.into_iter().next().unwrap().values;

            let opt_root_mails = self.cache.read().await.get_mails_core(&root_mails).await?;

            if opt_root_mails.missing.is_empty() {
                let root_mails_core = root_mails
                    .into_iter()
                    .map(|id| opt_root_mails.value.get(&id).cloned().unwrap())
                    .collect();
                return Ok(root_mails_core);
            } else {
                let missing_mails_core = self
                    .remote
                    .fetch_mails_core(&opt_root_mails.missing)
                    .await?;

                let mut cache_lock = self.cache.write().await;
                if let Some(current_email_get_state) = cache_lock.get_mail_state().await {
                    if *current_email_get_state != missing_mails_core.state {
                        self.apply_email_get_changes(&mut cache_lock).await?;
                    }
                }

                cache_lock
                    .upsert_mails_core(missing_mails_core.values.into_iter().collect())
                    .await?;

                let result = cache_lock.get_mails_core(&root_mails).await?;

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
        } = self.remote.fetch_root_mails(&id, &window).await?;

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

        cache_lock.insert_root_mails(&id, cache_root_mails).await?;

        cache_lock.upsert_mails_core(root_mails.clone()).await?;

        cache_lock
            .set_root_mails_state(&id, root_mails_query_state)
            .await?;

        let root_mails_core = root_mails.into_iter().map(|(_id, data)| data).collect();

        Ok(root_mails_core)
    }
}
