use crate::{
    datasource::types::{QueryWindow, remote},
    repository::Repository,
    types::{
        AccountId, MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailId,
        MailboxId,
    },
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

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
        window: QueryWindow,
        calculate_total: bool,

        tx: oneshot::Sender<color_eyre::Result<(Vec<MailDataCore>, Option<usize>)>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
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
    pub async fn get_mail_core(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataCore> {
        let _entry = self.mail_locks.get_mail_core.lock().await;

        match self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail_core(&id)
            .await?
        {
            Some(data) => Ok(data),
            None => {
                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail_core(id.clone())
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mails_core(vec![(id, result.value.clone())])
                    .await?;

                Ok(result.value)
            }
        }
    }

    pub async fn get_mail_preview(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataPreview> {
        let _enter = self.mail_locks.get_mail_preview.lock().await;

        match self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail_preview(&id)
            .await?
        {
            Some(data) => Ok(data),
            None => {
                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail_preview(id.clone())
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mails_preview(vec![(id, result.value.clone())])
                    .await?;

                Ok(result.value)
            }
        }
    }

    pub async fn get_mail_text_body(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataTextBody> {
        let _enter = self.mail_locks.get_mail_text_body.lock().await;

        let opt_text_body = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail_text_body(&id)
            .await?;

        match opt_text_body {
            Some(text_body) => Ok(text_body),
            None => {
                let remote::GetOneResult {
                    value: text_body,
                    state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail_text_body(&id)
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;
                self.ensure_email_changes(&account_id, &state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mail_text_body(&id, text_body.clone())
                    .await?;

                Ok(text_body)
            }
        }
    }

    pub async fn get_mail_html_body(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataHtmlBody> {
        let _enter = self.mail_locks.get_mail_html_body.lock().await;

        let opt_html_body = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail_html_body(&id)
            .await?;

        match opt_html_body {
            Some(html_body) => Ok(html_body),
            None => {
                let remote::GetOneResult {
                    value: html_body,
                    state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail_html_body(&id)
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mail_html_body(&id, html_body.clone())
                    .await?;

                Ok(html_body)
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        account_id: AccountId,
        id: MailboxId,
        window: QueryWindow,
        calculate_total: bool,
    ) -> color_eyre::Result<(Vec<MailDataCore>, Option<usize>)> {
        let _enter = self.mail_locks.query_root_mails.lock().await;

        let opt_root_mail_ids = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .query_root_mails(&id, window.clone())
            .await?;

        if let Some(root_mails) = opt_root_mail_ids
            && root_mails.missing.is_empty()
        {
            debug_assert_eq!(root_mails.values.len(), 1, "Full window was loaded");
            let root_mails = root_mails.values.into_iter().next().unwrap().values;

            let opt_root_mails_data = self
                .caches
                .get(&account_id)
                .unwrap()
                .read()
                .await
                .get_mails_core(&root_mails)
                .await?;

            if opt_root_mails_data.missing.is_empty() {
                let root_mails_data = root_mails
                    .into_iter()
                    .map(|id| opt_root_mails_data.value.get(&id).cloned().unwrap())
                    .collect();

                let total = self
                    .caches
                    .get(&account_id)
                    .unwrap()
                    .read()
                    .await
                    .calculate_total_root_mails(&id)
                    .await?;

                return Ok((root_mails_data, total));
            } else {
                let missing_mails_data = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mails_core(&opt_root_mails_data.missing)
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &missing_mails_data.state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mails_core(missing_mails_data.values.into_iter().collect())
                    .await?;

                let result = cache_lock.get_mails_core(&root_mails).await?;

                debug_assert!(result.missing.is_empty());

                let root_mails_core = root_mails
                    .into_iter()
                    .map(|id| result.value.get(&id).cloned().unwrap())
                    .collect();

                let total = cache_lock.calculate_total_root_mails(&id).await?;

                return Ok((root_mails_core, total));
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
            total,
        } = self
            .remote
            .get_remote_account(account_id.clone())
            .fetch_root_mails(&id, &window, calculate_total)
            .await?;

        let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

        self.ensure_email_changes(&account_id, &email_get_state, &mut cache_lock)
            .await?;

        self.ensure_root_mail_changes(&account_id, &id, &root_mails_query_state, &mut cache_lock)
            .await?;

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

        let root_mails = root_mails.into_iter().map(|(_id, data)| data).collect();

        Ok((root_mails, total))
    }
}
