pub mod mail;
pub mod mailbox;
pub mod thread;

use std::collections::HashMap;

use crate::{
    datasource::{
        Cache, Remote,
        types::{cache, remote},
    },
    types::{AccountId, MailId, MailboxId},
};
use tokio::sync::{RwLock, RwLockWriteGuard, mpsc};

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    Mail(mail::Command<C, R>),
    Mailbox(mailbox::Command<C, R>),
    Thread(thread::Command<C, R>),
    Quit,
}

#[derive(thiserror::Error, Debug)]
pub enum Error<C, R>
where
    C: Cache,
    R: Remote,
{
    #[error("Error from cache: {0}")]
    Cache(C::Error),

    #[error("Remote error: {0}")]
    Remote(R::Error),
}

pub struct Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    cache: RwLock<C>,
    remote: R,
    receiver: mpsc::Receiver<Command<C, R>>,
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub fn new(cache: C, remote: R, receiver: mpsc::Receiver<Command<C, R>>) -> Self {
        Self {
            cache: RwLock::new(cache),
            remote,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::Mail(cmd) => match cmd {
                    mail::Command::GetCore { id, tx } => {
                        let _ = tx.send(self.get_mail_core(id).await);
                    }
                    mail::Command::GetPreview { id, tx } => {
                        let _ = tx.send(self.get_mail_preview(id).await);
                    }
                    mail::Command::GetTextBody { id, tx } => {
                        let _ = tx.send(self.get_mail_text_body(id).await);
                    }
                    mail::Command::GetHtmlBody { id, tx } => {
                        let _ = tx.send(self.get_mail_html_body(id).await);
                    }
                    mail::Command::QueryRootMails {
                        mailbox,
                        start,
                        limit,
                        tx,
                    } => {
                        let _ = tx.send(self.query_root_mails(mailbox, start, limit).await);
                    }
                },
                Command::Mailbox(cmd) => match cmd {
                    mailbox::Command::GetChildren { id, tx } => {
                        let _ = tx.send(self.get_mailbox_children(id).await);
                    }
                },
                Command::Thread(cmd) => match cmd {
                    thread::Command::GetThread { id, tx } => {
                        let _ = tx.send(self.get_thread(id).await);
                    }
                },
                Command::Quit => self.quit(),
            }
        }
    }

    fn quit(&mut self) {
        self.receiver.close();
    }

    async fn apply_email_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        let Some(mut current_state) = cache_lock.get_mail_state().await.cloned() else {
            // no updates to do if there's no data :D
            return Ok(());
        };

        loop {
            let result = self
                .remote
                .fetch_mail_changes(&current_state)
                .await
                .map_err(Error::Remote)?;

            if !result.updated.is_empty() {
                // PERFORMANCE: join them all instead awaiting them sequentially
                let updated_mail_core_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_datas,
                        ..
                    } = cache_lock
                        .get_mails_core(&result.updated)
                        .await
                        .map_err(Error::Cache)?;

                    cached_datas.into_iter().map(|(id, _data)| id).collect()
                };
                let updated_mail_preview_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_datas,
                        ..
                    } = cache_lock
                        .get_mails_preview(&result.updated)
                        .await
                        .map_err(Error::Cache)?;

                    cached_datas.into_iter().map(|(id, _data)| id).collect()
                };
                let updated_mail_text_body_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_text_bodies,
                        ..
                    } = cache_lock
                        .get_mails_text_body(result.updated.clone())
                        .await
                        .map_err(Error::Cache)?;

                    cached_text_bodies
                        .into_iter()
                        .map(|(id, _content)| id)
                        .collect()
                };
                let updated_mail_html_body_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cache_html_bodies,
                        ..
                    } = cache_lock
                        .get_mails_html_body(result.updated.clone())
                        .await
                        .map_err(Error::Cache)?;

                    cache_html_bodies
                        .into_iter()
                        .map(|(id, _html_body)| id)
                        .collect()
                };

                let remote::GetOneResult {
                    value:
                        (
                            updated_mails_core,
                            updated_mails_preview,
                            updated_text_bodies,
                            updated_html_bodies,
                        ),
                    // TODO: Maybe check if this state is also the same? Otherwise => do more `/changes` request
                    state: _,
                } = self
                    .remote
                    .fetch_mail_updates(
                        updated_mail_core_ids,
                        updated_mail_preview_ids,
                        updated_mail_text_body_ids,
                        updated_mail_html_body_ids,
                    )
                    .await
                    .map_err(Error::Remote)?;

                // PERFORMANCE: put in `join` instead of sequentially
                cache_lock
                    .upsert_mails_core(updated_mails_core)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_preview(updated_mails_preview)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_text_body(updated_text_bodies)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_html_body(updated_html_bodies)
                    .await
                    .map_err(Error::Cache)?;
            };

            cache_lock
                .evict_mails(result.destroyed)
                .await
                .map_err(Error::Cache)?;

            current_state = result.new_state;
            cache_lock
                .set_mail_state(current_state.clone())
                .await
                .map_err(Error::Cache)?;

            if !result.has_more_changes {
                break;
            }
        }

        Ok(())
    }

    async fn apply_root_mail_query_changes(
        &self,
        id: &MailboxId,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        let Some(current_state) = cache_lock.get_root_mails_state(id).await.cloned() else {
            return Ok(());
        };

        let up_to_id = cache_lock.get_root_mails_last_id(id).await;

        let result = self
            .remote
            .fetch_root_mails_changes(id, &current_state, up_to_id.as_ref())
            .await
            .map_err(Error::Remote)?;

        cache_lock
            .evict_root_mails(id, result.removed.into_iter().collect())
            .await
            .map_err(Error::Cache)?;

        cache_lock
            .insert_root_mails(id, result.added)
            .await
            .map_err(Error::Cache)?;

        cache_lock
            .set_root_mails_state(id, result.new_state)
            .await
            .map_err(Error::Cache)?;

        Ok(())
    }

    async fn apply_mailbox_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        let Some(mut current_state) = cache_lock.get_mailbox_state().await.cloned() else {
            return Ok(());
        };

        todo!()
    }

    async fn apply_thread_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        let Some(mut current_state) = cache_lock.get_thread_state().await.cloned() else {
            return Ok(());
        };

        todo!()
    }
}
