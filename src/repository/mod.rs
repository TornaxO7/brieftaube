pub mod mail;
pub mod mailbox;
pub mod thread;

use crate::{
    datasource::{
        Cache, Remote,
        types::{QueryWindow, cache, remote},
    },
    types::{MailId, MailboxId},
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
                    mail::Command::QueryRootMails {
                        mailbox,
                        start,
                        limit,
                        tx,
                    } => {
                        let _ = tx.send(self.query_root_mails(mailbox, start, limit).await);
                    }
                    mail::Command::GetTextBody { id, tx } => {
                        let _ = tx.send(self.get_mail_text_body(id).await);
                    }
                    mail::Command::GetHtmlBody { id, tx } => {
                        let _ = tx.send(self.get_mail_html_body(id).await);
                    }
                    mail::Command::GetAttachments { id, tx } => {
                        let _ = tx.send(self.get_mail_attachments(id).await);
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
        let mut current_state = cache_lock
            .get_mail_state()
            .await
            .cloned()
            .expect("Why is it... None?");

        loop {
            let result = self
                .remote
                .fetch_mail_changes(&current_state)
                .await
                .map_err(Error::Remote)?;

            if !result.updated.is_empty() {
                // PERFORMANCE: join them all instead awaiting them sequentially
                let updated_mail_data_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_datas,
                        ..
                    } = cache_lock
                        .get_mails(&result.updated)
                        .await
                        .map_err(Error::Cache)?;

                    cached_datas.into_iter().map(|data| data.id).collect()
                };
                let updated_mail_text_body_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_text_bodies,
                        ..
                    } = cache_lock
                        .get_mails_text_body(&result.updated)
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
                        .get_mails_html_body(&result.updated)
                        .await
                        .map_err(Error::Cache)?;

                    cache_html_bodies
                        .into_iter()
                        .map(|(id, _html_body)| id)
                        .collect()
                };
                let updated_mail_attachments_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_attachments,
                        ..
                    } = cache_lock
                        .get_mails_attachments(&result.updated)
                        .await
                        .map_err(Error::Cache)?;

                    cached_attachments
                        .into_iter()
                        .map(|(id, _attachments)| id)
                        .collect()
                };

                let remote::GetOneResult {
                    value:
                        (
                            updated_mail_datas,
                            updated_text_bodies,
                            updated_html_bodies,
                            updated_attachments,
                        ),
                    state: _,
                } = self
                    .remote
                    .fetch_mail_updates(
                        &updated_mail_data_ids,
                        &updated_mail_text_body_ids,
                        &updated_mail_html_body_ids,
                        &updated_mail_attachments_ids,
                    )
                    .await
                    .map_err(Error::Remote)?;

                cache_lock
                    .upsert_mails(updated_mail_datas)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_text_body(&updated_text_bodies)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_html_body(&updated_html_bodies)
                    .await
                    .map_err(Error::Cache)?;
                cache_lock
                    .upsert_mails_attachments(&updated_attachments)
                    .await
                    .map_err(Error::Cache)?;
            };

            cache_lock
                .evict_mails(&result.destroyed)
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
        window: &QueryWindow,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        todo!()
    }

    async fn apply_mailbox_get_changes(
        &self,

        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        todo!()
    }

    async fn apply_thread_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, C>,
    ) -> Result<(), Error<C, R>> {
        todo!()
    }
}
