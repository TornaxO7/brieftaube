pub mod mail;
pub mod mailbox;
pub mod thread;

use crate::{
    datasource::{
        Cache, RemoteSession,
        types::{cache, remote},
    },
    types::{AccountId, MailId, MailboxId},
};
use std::collections::HashMap;
use tokio::sync::{RwLock, RwLockWriteGuard, mpsc};

#[derive(Debug)]
pub enum Command {
    Mail(mail::Command),
    Mailbox(mailbox::Command),
    Thread(thread::Command),
    Quit,
}

struct Repository {
    caches: HashMap<AccountId, RwLock<Box<dyn Cache>>>,
    remote: Box<dyn RemoteSession>,
    rx: mpsc::Receiver<Command>,

    mail_locks: mail::Locks,
    mailbox_locks: mailbox::Locks,
    thread_locks: thread::Locks,
}

impl Repository {
    async fn run(
        caches: HashMap<AccountId, RwLock<Box<dyn Cache>>>,
        remote: Box<dyn RemoteSession>,
        rx: mpsc::Receiver<Command>,
    ) {
        let mut repo = Self {
            caches,
            remote,
            rx,
            mail_locks: mail::Locks::default(),
            mailbox_locks: mailbox::Locks::default(),
            thread_locks: thread::Locks::default(),
        };

        while let Some(command) = repo.rx.recv().await {
            match command {
                Command::Mail(cmd) => match cmd.kind {
                    mail::CommandKind::GetCore { id, tx } => {
                        let _ = tx.send(repo.get_mail_core(cmd.account_id, id).await);
                    }
                    mail::CommandKind::GetPreview { id, tx } => {
                        let _ = tx.send(repo.get_mail_preview(cmd.account_id, id).await);
                    }
                    mail::CommandKind::GetTextBody { id, tx } => {
                        let _ = tx.send(repo.get_mail_text_body(cmd.account_id, id).await);
                    }
                    mail::CommandKind::GetHtmlBody { id, tx } => {
                        let _ = tx.send(repo.get_mail_html_body(cmd.account_id, id).await);
                    }
                    mail::CommandKind::QueryRootMails {
                        mailbox,
                        window,
                        calculate_total,
                        tx,
                    } => {
                        let _ = tx.send(
                            repo.query_root_mails(cmd.account_id, mailbox, window, calculate_total)
                                .await,
                        );
                    }
                },
                Command::Mailbox(cmd) => match cmd.kind {
                    mailbox::CommandKind::GetChildren { id, tx } => {
                        let _ = tx.send(repo.get_mailbox_children(cmd.account_id, id).await);
                    }
                },
                Command::Thread(cmd) => match cmd.kind {
                    thread::CommandKind::GetThread { id, tx } => {
                        let _ = tx.send(repo.get_thread(cmd.account_id, id).await);
                    }
                },
                Command::Quit => repo.quit(),
            }
        }
    }

    fn quit(&mut self) {
        self.rx.close();
    }

    async fn apply_email_get_changes(
        &self,
        account_id: &AccountId,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        let Some(mut current_state) = cache_lock.get_mail_state().await.cloned() else {
            // no updates to do if there's no data :D
            return Ok(());
        };

        loop {
            let result = self
                .remote
                .get_remote_account(account_id.clone())
                .fetch_mail_changes(&current_state)
                .await?;

            if !result.updated.is_empty() {
                // PERFORMANCE: join them all instead awaiting them sequentially
                let updated_mail_core_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_datas,
                        ..
                    } = cache_lock.get_mails_core(&result.updated).await?;

                    cached_datas.into_iter().map(|(id, _data)| id).collect()
                };
                let updated_mail_preview_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_datas,
                        ..
                    } = cache_lock.get_mails_preview(&result.updated).await?;

                    cached_datas.into_iter().map(|(id, _data)| id).collect()
                };
                let updated_mail_text_body_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cached_text_bodies,
                        ..
                    } = cache_lock.get_mails_text_body(&result.updated).await?;

                    cached_text_bodies
                        .into_iter()
                        .map(|(id, _content)| id)
                        .collect()
                };
                let updated_mail_html_body_ids: Vec<MailId> = {
                    let cache::GetBatchResult {
                        value: cache_html_bodies,
                        ..
                    } = cache_lock.get_mails_html_body(&result.updated).await?;

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
                    .get_remote_account(account_id.clone())
                    .fetch_mail_updates(
                        &updated_mail_core_ids,
                        &updated_mail_preview_ids,
                        &updated_mail_text_body_ids,
                        &updated_mail_html_body_ids,
                    )
                    .await?;

                // PERFORMANCE: put in `join` instead of sequentially
                cache_lock.upsert_mails_core(updated_mails_core).await?;
                cache_lock
                    .upsert_mails_preview(updated_mails_preview)
                    .await?;
                cache_lock
                    .upsert_mails_text_body(&updated_text_bodies)
                    .await?;
                cache_lock
                    .upsert_mails_html_body(&updated_html_bodies)
                    .await?;
            };

            cache_lock.evict_mails(&result.destroyed).await?;

            current_state = result.new_state;
            cache_lock.set_mail_state(current_state.clone()).await?;

            if !result.has_more_changes {
                break;
            }
        }

        Ok(())
    }

    async fn apply_root_mail_query_changes(
        &self,
        account_id: &AccountId,
        id: &MailboxId,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        let Some(current_state) = cache_lock.get_root_mails_state(id).await.cloned() else {
            return Ok(());
        };

        let up_to_id = cache_lock.get_root_mails_last_id(id).await;

        let result = self
            .remote
            .get_remote_account(account_id.clone())
            .fetch_root_mails_changes(id, &current_state, up_to_id.as_ref())
            .await?;

        cache_lock
            .evict_root_mails(id, result.removed.into_iter().collect())
            .await?;

        cache_lock.insert_root_mails(id, result.added).await?;

        cache_lock
            .set_root_mails_state(id, result.new_state)
            .await?;

        Ok(())
    }

    async fn apply_mailbox_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        let Some(mut _current_state) = cache_lock.get_mailbox_state().await.cloned() else {
            return Ok(());
        };

        todo!()
    }

    async fn apply_thread_get_changes(
        &self,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        let Some(mut _current_state) = cache_lock.get_thread_state().await.cloned() else {
            return Ok(());
        };

        todo!()
    }
}

#[derive(Clone)]
pub struct RepositoryHandler {
    tx: mpsc::Sender<Command>,
}

impl RepositoryHandler {
    pub fn new(
        caches: HashMap<AccountId, RwLock<Box<dyn Cache>>>,
        remote: Box<dyn RemoteSession>,
    ) -> RepositoryHandler {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(Repository::run(caches, remote, rx));

        Self { tx }
    }

    pub async fn execute(&self, command: Command) {
        self.tx.send(command).await.unwrap();
    }
}
