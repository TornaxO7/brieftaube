use crate::{
    datasource::types::{QueryWindow, remote},
    repository::Repository,
    types::{
        AccountId, MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailDto,
        MailDtoCore, MailDtoHtmlPartIds, MailDtoPreview, MailDtoTextPartIds, MailId, MailboxId,
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

        tx: oneshot::Sender<color_eyre::Result<Vec<MailDataCore>>>,
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

        let opt_mail = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail(&id)
            .await?;

        match opt_mail {
            Some(data) => Ok(data.into()),
            None => {
                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id.clone(), MailDtoCore::GET_REQUEST_PROPERTIES.to_vec())
                    .await?;

                let mail_dto = MailDto::new(MailDtoCore::from(result.value));

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![mail_dto.clone()]).await?;

                Ok(mail_dto.into())
            }
        }
    }

    pub async fn get_mail_preview(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataPreview> {
        let _enter = self.mail_locks.get_mail_preview.lock().await;

        let mut cached_mail = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail(&id)
            .await?
            .expect("Core has been loaded before");

        match MailDataPreview::new(cached_mail.clone()) {
            Some(preview) => Ok(preview),
            None => {
                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id.clone(), MailDtoPreview::GET_REQUEST_PROPERTIES.to_vec())
                    .await?;

                cached_mail.preview = Some(MailDtoPreview::from(result.value));

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![cached_mail.clone()]).await?;

                Ok(MailDataPreview::new(cached_mail).expect("Just set `preview`"))
            }
        }
    }

    pub async fn get_mail_text_body(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataTextBody> {
        let _enter = self.mail_locks.get_mail_text_body.lock().await;

        let mut cached_mail = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail(&id)
            .await?
            .expect("Core fetched");

        match MailDataTextBody::new(cached_mail.clone()) {
            Some(text_body) => Ok(text_body),
            None => {
                let remote::GetOneResult {
                    value: jmap_mail,
                    state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id, MailDtoTextPartIds::GET_REQUEST_PROPERTIES.to_vec())
                    .await?;

                let (text_part_ids, text_body_parts) = MailDtoTextPartIds::new(jmap_mail);
                cached_mail.text_part_ids = Some(text_part_ids);
                cached_mail.body_parts.extend(text_body_parts);

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;
                self.ensure_email_changes(&account_id, &state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![cached_mail.clone()]).await?;

                Ok(MailDataTextBody::new(cached_mail).expect("Just added"))
            }
        }
    }

    pub async fn get_mail_html_body(
        &self,
        account_id: AccountId,
        id: MailId,
    ) -> color_eyre::Result<MailDataHtmlBody> {
        let _enter = self.mail_locks.get_mail_html_body.lock().await;

        let mut cached_mail = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail(&id)
            .await?
            .expect("Already fetched");

        match MailDataHtmlBody::new(cached_mail.clone()) {
            Some(html_body) => Ok(html_body),
            None => {
                let remote::GetOneResult {
                    value: jmap_mail,
                    state,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id, MailDtoHtmlPartIds::GET_REQUEST_PROPERTIES.to_vec())
                    .await?;

                let (html_part_ids, html_body_parts) = MailDtoHtmlPartIds::new(jmap_mail);
                cached_mail.html_part_ids = Some(html_part_ids);
                cached_mail.body_parts.extend(html_body_parts);

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![cached_mail.clone()]).await?;

                Ok(MailDataHtmlBody::new(cached_mail).expect("Just added"))
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        account_id: AccountId,
        id: MailboxId,
        window: QueryWindow,
    ) -> color_eyre::Result<Vec<MailDataCore>> {
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
            debug_assert_eq!(
                root_mails.values.len(),
                1,
                "Full query window is in cache. Got {} windows instead.",
                root_mails.values.len()
            );
            let root_mails = root_mails.values.into_iter().next().unwrap().values;

            let cached_root_mails = self
                .caches
                .get(&account_id)
                .unwrap()
                .read()
                .await
                .get_mails(&root_mails)
                .await?;

            if cached_root_mails.missing.is_empty() {
                return Ok(cached_root_mails
                    .value
                    .into_iter()
                    .map(MailDataCore::from)
                    .collect());
            } else {
                let missing_mails_data = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mails(
                        &cached_root_mails.missing,
                        MailDtoCore::GET_REQUEST_PROPERTIES.to_vec(),
                    )
                    .await?;

                debug_assert!(missing_mails_data.not_found.is_empty());

                let missing_root_mails: Vec<MailDto> = missing_mails_data
                    .values
                    .into_iter()
                    .map(|mail| MailDto::new(MailDtoCore::from(mail)))
                    .collect();

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_email_changes(&account_id, &missing_mails_data.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(missing_root_mails).await?;

                let result = cache_lock.get_mails(&root_mails).await?;

                debug_assert!(result.missing.is_empty());

                return Ok(result.value.into_iter().map(MailDataCore::from).collect());
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
            .get_remote_account(account_id.clone())
            .fetch_root_mails(&id, &window)
            .await?;

        let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

        self.ensure_email_changes(&account_id, &email_get_state, &mut cache_lock)
            .await?;

        self.ensure_root_mail_changes(&account_id, &id, &root_mails_query_state, &mut cache_lock)
            .await?;

        let root_mail_ids: Vec<(MailId, usize)> = root_mails
            .iter()
            .enumerate()
            .map(|(offset, root_mail)| {
                let idx = window.start as usize + offset;
                (root_mail.id.clone(), idx)
            })
            .collect();

        cache_lock.insert_root_mails(&id, root_mail_ids).await?;

        let root_mails: Vec<MailDto> = root_mails.into_iter().map(MailDto::new).collect();
        cache_lock.upsert_mails(root_mails.clone()).await?;
        Ok(root_mails.into_iter().map(MailDataCore::from).collect())
    }
}
