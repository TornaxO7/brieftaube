// mod database;
pub mod hashmap;
pub mod jmap;
pub mod types;

use crate::types::{
    AccountData, AccountId, BlobId, CachedMail, CachedMailbox, MailId, MailProperty, MailboxId,
    MailboxUpdate, ParentMailboxId, ThreadId,
};
use async_trait::async_trait;
use color_eyre::Result;
use std::collections::{HashMap, HashSet};
use types::{GetState, QueryState, QueryWindow, cache, remote};

pub trait Cache:
    MailCache + RootMailsCache + MailboxCache + ThreadCache + BlobCache + Send + Sync
{
}

pub trait RemoteSession: Send + Sync {
    fn get_accounts(&self) -> Vec<AccountData>;

    fn get_remote_account(&self, account_id: AccountId) -> Box<dyn RemoteAccount>;
}

pub trait RemoteAccount:
    MailRemote + RootMailsRemote + MailboxRemote + ThreadRemote + BlobRemote + Send + Sync
{
}

#[async_trait]
pub trait MailCache {
    async fn get_mail_state(&self) -> Option<&GetState>;

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<()>;

    async fn get_mail(&self, id: &MailId) -> Result<Option<CachedMail>>
    where
        Self: Sync,
    {
        let result = self.get_mails(&[id.clone()]).await?;

        if result.missing.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap()))
        } else {
            Ok(None)
        }
    }

    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<CachedMail>, Vec<MailId>>>;

    async fn upsert_mails(&mut self, mails: Vec<CachedMail>) -> Result<()>;

    async fn evict_mails(
        &mut self,
        mails: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<CachedMail>, Vec<MailId>>>;
}

#[async_trait]
pub trait MailRemote {
    async fn fetch_mail(
        &self,
        id: MailId,
        properties: Vec<MailProperty>,
    ) -> Result<remote::GetOneResult<CachedMail>> {
        let result = self.fetch_mails(&[id], properties).await?;
        debug_assert!(result.not_found.is_empty());

        Ok(remote::GetOneResult {
            value: result.values.into_iter().next().unwrap(),
            state: result.state,
        })
    }

    async fn fetch_mails(
        &self,
        ids: &[MailId],
        properties: Vec<MailProperty>,
    ) -> Result<remote::GetBatchResult<Vec<CachedMail>, Vec<MailId>>>;

    async fn destroy_mails(
        &self,
        ids: &[MailId],
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>>;

    async fn fetch_mail_changes(&self, since: &GetState)
    -> Result<remote::GetChangeResult<MailId>>;

    async fn create_mail(&self);
}

#[async_trait]
pub trait RootMailsCache: MailCache {
    async fn get_root_mails_state(&self, mailbox: &MailboxId) -> Option<&QueryState>;

    async fn set_root_mails_state(
        &mut self,
        mailbox: &MailboxId,
        new_state: QueryState,
    ) -> Result<()>;

    async fn get_root_mails_last_id(&self, mailbox: &MailboxId) -> Option<MailId>;

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<Option<cache::QueryResponse<MailId>>>;

    async fn insert_root_mails(
        &mut self,
        mailbox: &MailboxId,
        root_mails: Vec<(MailId, usize)>,
    ) -> Result<()>;

    async fn evict_root_mails(&mut self, mailbox: &MailboxId, ids: HashSet<MailId>) -> Result<()>;
}

#[async_trait]
pub trait RootMailsRemote: MailRemote {
    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: &QueryWindow,
        properties: Vec<MailProperty>,
    ) -> Result<remote::QueryResponse<remote::GetOneResult<Vec<CachedMail>>>>;

    async fn fetch_root_mails_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
        up_to_id: Option<&MailId>,
    ) -> Result<remote::QueryChangeResult<MailId>>;
}

#[async_trait]
pub trait MailboxCache {
    async fn get_mailbox_state(&self) -> Option<&GetState>;

    async fn get_mailbox(&self, id: MailboxId) -> Result<Option<CachedMailbox>>
    where
        Self: Sync,
    {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.value.into_iter().next())
    }

    async fn get_all_mailboxes(&self) -> Result<Option<Vec<CachedMailbox>>>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetBatchResult<Vec<CachedMailbox>, Vec<MailboxId>>>;

    async fn get_mailbox_children(
        &self,
        parent: &ParentMailboxId,
    ) -> Result<Option<Vec<CachedMailbox>>>;

    async fn upsert_mailboxes(
        &mut self,
        mailboxes: Vec<CachedMailbox>,
        state: GetState,
    ) -> Result<()>;

    async fn evict_mailboxes(&mut self, ids: &[MailboxId], new_state: GetState) -> Result<()>;
}

#[async_trait]
pub trait MailboxRemote {
    async fn fetch_mailboxes_all(&self) -> Result<remote::GetOneResult<Vec<CachedMailbox>>>;

    async fn fetch_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailboxId>>;

    // async fn create_mailbox(
    //     &self,
    //     new: MailboxNew,
    // ) -> Result<remote::CreateResult<jmap_client::mailbox::Mailbox>>;

    async fn update_mailboxes(
        &self,
        updates: Vec<(CachedMailbox, MailboxUpdate)>,
        since: &GetState,
    ) -> Result<remote::UpdateResult<MailboxId, CachedMailbox>>;

    async fn destroy_mailboxes(
        &self,
        ids: &[MailboxId],
        on_destroy_remove_emails: bool,
    ) -> Result<remote::DestroyResult<MailboxId>>;
}

#[async_trait]
pub trait ThreadCache {
    async fn get_thread_state(&self) -> Option<&GetState>;

    async fn set_thread_state(&mut self, new_state: GetState) -> Result<()>;

    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Vec<MailId>>> {
        let result = self.get_threads(&[id.clone()]).await?;

        if result.missing.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap().1))
        } else {
            Ok(None)
        }
    }

    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<cache::GetBatchResult<HashMap<ThreadId, Vec<MailId>>, Vec<ThreadId>>>;

    async fn upsert_thread(&mut self, id: ThreadId, mails: Vec<MailId>) -> Result<()> {
        self.upsert_threads(vec![(id, mails)]).await
    }

    async fn upsert_threads(&mut self, threads: Vec<(ThreadId, Vec<MailId>)>) -> Result<()>;

    async fn evict_threads(&mut self, ids: &[ThreadId]) -> Result<()>;
}

#[async_trait]
pub trait ThreadRemote {
    async fn fetch_thread(&self, id: &ThreadId) -> Result<remote::GetOneResult<Vec<MailId>>> {
        let result = self.fetch_threads(&[id.clone()]).await?;
        debug_assert!(result.not_found.is_empty());

        Ok(remote::GetOneResult {
            value: result.values.into_values().next().unwrap(),
            state: result.state,
        })
    }

    async fn fetch_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<remote::GetBatchResult<HashMap<ThreadId, Vec<MailId>>, Vec<ThreadId>>>;

    async fn fetch_thread_with_mails(
        &self,
        id: &ThreadId,
        properties: Vec<MailProperty>,
    ) -> Result<remote::GetOneResult<remote::GetOneResult<Vec<CachedMail>>>> {
        let result = self
            .fetch_threads_with_mails(&[id.clone()], properties)
            .await?;

        debug_assert!(result.not_found.is_empty());

        Ok(remote::GetOneResult {
            value: remote::GetOneResult {
                value: result.values.value.into_values().next().unwrap(),
                state: result.values.state,
            },
            state: result.state,
        })
    }

    async fn fetch_threads_with_mails(
        &self,
        ids: &[ThreadId],
        properties: Vec<MailProperty>,
    ) -> Result<
        remote::GetBatchResult<
            remote::GetOneResult<HashMap<ThreadId, Vec<CachedMail>>>,
            Vec<ThreadId>,
        >,
    >;

    async fn fetch_thread_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<ThreadId>>;
}

#[async_trait]
pub trait BlobCache {
    async fn get_blob(&self, id: &BlobId) -> Option<Vec<u8>>;

    async fn upsert_blob(&mut self, id: BlobId, blob: Vec<u8>) -> Result<()>;
}

#[async_trait]
pub trait BlobRemote {
    async fn fetch_blob(&self, id: BlobId) -> Result<Vec<u8>>;
}
