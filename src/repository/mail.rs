use crate::{
    datasource::types::{QueryWindow, cache, remote},
    repository::Repository,
    types::{AccountId, CachedMail, MailId, MailProperty, MailboxId},
};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    GetMail {
        id: MailId,
        properties: Vec<MailProperty>,
        tx: oneshot::Sender<color_eyre::Result<CachedMail>>,
    },
    QueryRootMails {
        mailbox: MailboxId,
        window: QueryWindow,
        properties: Vec<MailProperty>,

        tx: oneshot::Sender<color_eyre::Result<Vec<CachedMail>>>,
    },
    ComposeNewHtmlMail {
        id: MailId,
        tx: oneshot::Sender<color_eyre::Result<()>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::Mail(cmd)
    }
}

#[derive(Default)]
pub struct Locks {
    get_mail: Mutex<()>,
    query_root_mails: Mutex<()>,
}

impl Repository {
    pub async fn get_mail(
        &self,
        account_id: AccountId,
        id: MailId,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<CachedMail> {
        let _entry = self.mail_locks.get_mail.lock().await;

        let opt_cached_mail = self
            .caches
            .get(&account_id)
            .unwrap()
            .read()
            .await
            .get_mail(&id)
            .await?;

        match opt_cached_mail {
            Some(mut cached_mail) => {
                if cached_mail.has_properties(&properties) {
                    return Ok(cached_mail);
                }

                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id, properties)
                    .await?;

                cached_mail.merge(result.value);

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![cached_mail.clone()]).await?;

                Ok(cached_mail)
            }
            None => {
                let result = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mail(id, properties)
                    .await?;

                let cached_mail = result.value;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &result.state, &mut cache_lock)
                    .await?;

                cache_lock.upsert_mails(vec![cached_mail.clone()]).await?;

                Ok(cached_mail)
            }
        }
    }

    pub async fn query_root_mails(
        &self,
        account_id: AccountId,
        id: MailboxId,
        window: QueryWindow,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<Vec<CachedMail>> {
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

            let cache::GetBatchResult {
                value: mut cached_root_mails,
                missing: missing_cached_root_mails,
            } = self
                .caches
                .get(&account_id)
                .unwrap()
                .read()
                .await
                .get_mails(&root_mails)
                .await?;

            if missing_cached_root_mails.is_empty() {
                if cached_root_mails
                    .iter()
                    .all(|cached_mail| cached_mail.has_properties(&properties))
                {
                    return Ok(cached_root_mails);
                }

                let cached_root_mail_ids_with_missing_properties: Vec<MailId> = cached_root_mails
                    .iter()
                    .filter_map(|cached_root_mail| {
                        if cached_root_mail.has_properties(&properties) {
                            None
                        } else {
                            Some(cached_root_mail.id.clone())
                        }
                    })
                    .collect();

                let fetched_root_mails = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mails(&cached_root_mail_ids_with_missing_properties, properties)
                    .await?;

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &fetched_root_mails.state, &mut cache_lock)
                    .await?;

                for fetched_root_mail in fetched_root_mails.values {
                    let cached_root_mail = cached_root_mails
                        .iter_mut()
                        .find(|cached_root_mail| cached_root_mail.id == fetched_root_mail.id)
                        .unwrap();
                    cached_root_mail.merge(fetched_root_mail);
                }

                cache_lock.upsert_mails(cached_root_mails.clone()).await?;
                return Ok(cached_root_mails);
            } else {
                // TODO: Well, but what happens, if the cached mails are missing some properties?
                let fetched_root_mails = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mails(&missing_cached_root_mails, properties)
                    .await?;

                debug_assert!(fetched_root_mails.not_found.is_empty());

                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &fetched_root_mails.state, &mut cache_lock)
                    .await?;

                cache_lock
                    .upsert_mails(fetched_root_mails.values.clone())
                    .await?;

                let cached_root_mails = cache_lock.get_mails(&root_mails).await?;
                debug_assert!(cached_root_mails.missing.is_empty(), "We just fetched them");

                return Ok(cached_root_mails.value);
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
            .fetch_root_mails(&id, &window, properties)
            .await?;

        let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

        self.ensure_mail_changes(&account_id, &email_get_state, &mut cache_lock)
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

        let root_mails: Vec<CachedMail> = root_mails.into_iter().map(CachedMail::from).collect();
        cache_lock.upsert_mails(root_mails.clone()).await?;
        Ok(root_mails)
    }

    pub async fn compose_new_html_mail(
        &self,
        _account_id: AccountId,
        _mail_id: MailId,
    ) -> color_eyre::Result<()> {
        todo!()
    }
}
