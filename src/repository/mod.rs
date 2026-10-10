pub mod blob;
pub mod mail;
pub mod mailbox;
pub mod thread;

use crate::{
    datasource::{
        Cache, RemoteSession,
        types::{GetState, QueryState, QueryWindow, cache, remote},
    },
    types::{
        AccountId, BlobId, CachedMail, InitMailboxData, MailId, MailProperty, MailboxId,
        ParentMailboxId, ThreadId,
    },
};
use std::collections::HashMap;
use tokio::sync::{RwLock, RwLockWriteGuard, mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Mail(mail::Command),
    Mailbox(mailbox::Command),
    Thread(thread::Command),
    Blob(blob::Command),
    Quit,
}

struct Repository {
    caches: HashMap<AccountId, RwLock<Box<dyn Cache>>>,
    remote: Box<dyn RemoteSession>,
    rx: mpsc::Receiver<Command>,

    mail_locks: mail::Locks,
    mailbox_locks: mailbox::Locks,
    thread_locks: thread::Locks,
    blob_locks: blob::Locks,
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
            blob_locks: blob::Locks::default(),
        };

        while let Some(command) = repo.rx.recv().await {
            match command {
                Command::Mail(cmd) => match cmd.kind {
                    // mail::CommandKind::GetCore { id, tx } => {
                    //     let _ = tx.send(repo.get_mail_core(cmd.account_id, id).await);
                    // }
                    // mail::CommandKind::GetPreview { id, tx } => {
                    //     let _ = tx.send(repo.get_mail_preview(cmd.account_id, id).await);
                    // }
                    // mail::CommandKind::GetTextBody { id, tx } => {
                    //     let _ = tx.send(repo.get_mail_text_body(cmd.account_id, id).await);
                    // }
                    // mail::CommandKind::GetHtmlBody { id, tx } => {
                    //     let _ = tx.send(repo.get_mail_html_body(cmd.account_id, id).await);
                    // }
                    mail::CommandKind::GetMail { id, properties, tx } => {
                        let _ = tx.send(repo.get_mail(cmd.account_id, id, properties).await);
                    }
                    mail::CommandKind::QueryRootMails {
                        mailbox,
                        window,
                        properties,
                        tx,
                    } => {
                        let _ = tx.send(
                            repo.query_root_mails(cmd.account_id, mailbox, window, properties)
                                .await,
                        );
                    }
                    mail::CommandKind::ComposeNewHtmlMail { id, tx } => {
                        let _ = tx.send(repo.compose_new_html_mail(cmd.account_id, id).await);
                    }
                },
                Command::Mailbox(cmd) => match cmd.kind {
                    // mailbox::CommandKind::GetChildren { id, tx } => {
                    //     let _ = tx.send(repo.get_mailbox_children(cmd.account_id, id).await);
                    // }
                    mailbox::CommandKind::Init {
                        id,
                        amount_init_mails,
                        mail_properties,
                        tx,
                    } => {
                        let _ = tx.send(
                            repo.get_init_mailbox_data(
                                cmd.account_id,
                                id,
                                amount_init_mails,
                                mail_properties,
                            )
                            .await,
                        );
                    }
                },
                Command::Thread(cmd) => match cmd.kind {
                    thread::CommandKind::GetThread { id, properties, tx } => {
                        let _ = tx.send(repo.get_thread(cmd.account_id, id, properties).await);
                    }
                },
                Command::Blob(cmd) => match cmd.kind {
                    blob::CommandKind::GetBlob { id, tx } => {
                        let _ = tx.send(repo.get_blob(cmd.account_id, id).await);
                    }
                },
                Command::Quit => repo.quit(),
            }
        }
    }

    fn quit(&mut self) {
        self.rx.close();
    }

    async fn ensure_mail_changes(
        &self,
        account_id: &AccountId,
        new_state: &GetState,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        match cache_lock.get_mail_state().await {
            Some(current_state) => {
                if current_state != new_state {
                    self.apply_email_get_changes(account_id, cache_lock).await?;
                }

                debug_assert_eq!(cache_lock.get_mail_state().await.unwrap(), new_state);
                Ok(())
            }
            None => cache_lock.set_mail_state(new_state.clone()).await,
        }
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
                let mut cached_mails = cache_lock.get_mails(&result.updated).await?.value;

                let updated_mail_ids: Vec<MailId> = cached_mails
                    .iter()
                    .map(|cached_mail| cached_mail.id.clone())
                    .collect();

                let remote::GetBatchResult {
                    values: updated_mails,
                    not_found,
                    state: _,
                } = self
                    .remote
                    .get_remote_account(account_id.clone())
                    .fetch_mails(
                        &updated_mail_ids,
                        vec![MailProperty::MailboxIds, MailProperty::Keywords],
                    )
                    .await?;
                debug_assert!(not_found.is_empty());

                for updated_mail in updated_mails {
                    let cached_mail = cached_mails
                        .iter_mut()
                        .find(|cached_mail| cached_mail.id == updated_mail.id)
                        .unwrap();

                    cached_mail.merge(updated_mail);
                }

                cache_lock.upsert_mails(cached_mails).await?;
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

    async fn ensure_root_mail_changes(
        &self,
        account_id: &AccountId,
        id: &MailboxId,
        state: &QueryState,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        match cache_lock.get_root_mails_state(id).await {
            Some(current_state) => {
                if current_state != state {
                    self.apply_root_mail_query_changes(account_id, id, cache_lock)
                        .await?;
                }

                debug_assert_eq!(cache_lock.get_root_mails_state(id).await.unwrap(), state);
                Ok(())
            }
            None => cache_lock.set_root_mails_state(id, state.clone()).await,
        }
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

    async fn ensure_thread_changes(
        &self,
        account_id: &AccountId,
        state: &GetState,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        match cache_lock.get_thread_state().await {
            Some(current_state) => {
                if current_state != state {
                    self.apply_thread_get_changes(account_id, cache_lock)
                        .await?;
                }

                debug_assert_eq!(cache_lock.get_thread_state().await.unwrap(), state);
                Ok(())
            }
            None => cache_lock.set_thread_state(state.clone()).await,
        }
    }

    async fn apply_thread_get_changes(
        &self,
        account_id: &AccountId,
        cache_lock: &mut RwLockWriteGuard<'_, Box<dyn Cache>>,
    ) -> color_eyre::Result<()> {
        let Some(mut current_state) = cache_lock.get_thread_state().await.cloned() else {
            return Ok(());
        };

        let remote = self.remote.get_remote_account(account_id.clone());

        loop {
            let changes = remote.fetch_thread_changes(&current_state).await?;

            if !changes.updated.is_empty() {
                let cache::GetBatchResult {
                    value: cached_thread_ids,
                    ..
                } = cache_lock.get_threads(&changes.updated).await?;

                let cached_thread_ids: Vec<ThreadId> = cached_thread_ids.into_keys().collect();

                let remote::GetBatchResult {
                    values: threads_with_mail_ids,
                    state: new_thread_get_state,
                    ..
                } = remote.fetch_threads(&cached_thread_ids).await?;

                cache_lock
                    .upsert_threads(threads_with_mail_ids.into_iter().collect())
                    .await?;

                cache_lock.set_thread_state(new_thread_get_state).await?;
            }

            cache_lock.evict_threads(&changes.destroyed).await?;

            current_state = changes.new_state;
            cache_lock.set_thread_state(current_state.clone()).await?;

            if !changes.has_more_changes {
                break;
            }
        }

        Ok(())
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

    async fn execute<T>(&self, into_command: impl FnOnce(oneshot::Sender<T>) -> Command) -> T {
        let (tx, rx) = oneshot::channel();
        self.tx.send(into_command(tx)).await.unwrap();
        rx.await.expect("`tx` didn't drop first")
    }

    pub async fn quit(&self) {
        self.execute::<()>(|_| Command::Quit).await;
    }

    pub async fn get_init_mailbox(
        &self,
        account_id: AccountId,
        id: ParentMailboxId,
        amount_init_mails: usize,
        mail_properties: Vec<MailProperty>,
    ) -> color_eyre::Result<InitMailboxData> {
        self.execute(|tx| {
            mailbox::Command {
                account_id,
                kind: mailbox::CommandKind::Init {
                    id,
                    amount_init_mails,
                    mail_properties,
                    tx,
                },
            }
            .into()
        })
        .await
    }

    pub async fn get_mail(
        &self,
        account_id: AccountId,
        id: MailId,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<CachedMail> {
        self.execute(|tx| {
            mail::Command {
                account_id,
                kind: mail::CommandKind::GetMail { id, properties, tx },
            }
            .into()
        })
        .await
    }

    // pub async fn get_mail_core(
    //     &self,
    //     account_id: AccountId,
    //     mail_id: MailId,
    // ) -> color_eyre::Result<MailDataCore> {
    //     self.execute(|tx| {
    //         mail::Command {
    //             account_id,
    //             kind: mail::CommandKind::GetCore { id: mail_id, tx },
    //         }
    //         .into()
    //     })
    //     .await
    // }

    pub async fn query_mails(
        &self,
        account_id: AccountId,
        mailbox: MailboxId,
        window: QueryWindow,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<Vec<CachedMail>> {
        self.execute(|tx| {
            mail::Command {
                account_id,
                kind: mail::CommandKind::QueryRootMails {
                    mailbox,
                    window,
                    properties,
                    tx,
                },
            }
            .into()
        })
        .await
    }

    pub async fn get_thread_mails(
        &self,
        account_id: AccountId,
        thread_id: ThreadId,
        properties: Vec<MailProperty>,
    ) -> color_eyre::Result<Vec<CachedMail>> {
        self.execute(|tx| {
            thread::Command {
                account_id,
                kind: thread::CommandKind::GetThread {
                    id: thread_id,
                    tx,
                    properties,
                },
            }
            .into()
        })
        .await
    }

    // pub async fn get_mail_preview(
    //     &self,
    //     account_id: AccountId,
    //     mail_id: MailId,
    // ) -> color_eyre::Result<MailDataPreview> {
    //     self.execute(|tx| {
    //         mail::Command {
    //             account_id,
    //             kind: mail::CommandKind::GetPreview { id: mail_id, tx },
    //         }
    //         .into()
    //     })
    //     .await
    // }

    // pub async fn get_mail_text_body(
    //     &self,
    //     account_id: AccountId,
    //     mail_id: MailId,
    // ) -> color_eyre::Result<MailDataTextBody> {
    //     self.execute(|tx| {
    //         mail::Command {
    //             account_id,
    //             kind: mail::CommandKind::GetTextBody { id: mail_id, tx },
    //         }
    //         .into()
    //     })
    //     .await
    // }

    // pub async fn get_mail_html_body(
    //     &self,
    //     account_id: AccountId,
    //     mail_id: MailId,
    // ) -> color_eyre::Result<MailDataHtmlBody> {
    //     self.execute(|tx| {
    //         mail::Command {
    //             account_id,
    //             kind: mail::CommandKind::GetHtmlBody { id: mail_id, tx },
    //         }
    //         .into()
    //     })
    //     .await
    // }

    pub async fn get_blob(
        &self,
        account_id: AccountId,
        blob_id: BlobId,
    ) -> color_eyre::Result<Vec<u8>> {
        self.execute(|tx| {
            blob::Command {
                account_id,
                kind: blob::CommandKind::GetBlob { id: blob_id, tx },
            }
            .into()
        })
        .await
    }

    pub async fn compose_new_html_mail(
        &self,
        account_id: AccountId,
        mail_id: MailId,
    ) -> color_eyre::Result<()> {
        self.execute(|tx| {
            mail::Command {
                account_id,
                kind: mail::CommandKind::ComposeNewHtmlMail { id: mail_id, tx },
            }
            .into()
        })
        .await
    }
}
