// mod database;
pub mod hashmap;
pub mod jmap;
pub mod types;

use crate::types::{
    AccountData, AccountId, MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody,
    MailId, MailboxData, MailboxId, MailboxNew, MailboxUpdate, ParentMailboxId, ThreadId,
};
use async_trait::async_trait;
use color_eyre::Result;
use std::collections::{HashMap, HashSet};
use types::{GetState, QueryState, QueryWindow, cache, remote};

pub trait Cache: MailCache + RootMailsCache + MailboxCache + ThreadCache + Send + Sync {}

pub trait RemoteSession: Send + Sync {
    fn get_accounts(&self) -> Vec<AccountData>;

    fn get_remote_account(&self, account_id: AccountId) -> Box<dyn RemoteAccount>;
}

pub trait RemoteAccount:
    MailRemote + RootMailsRemote + MailboxRemote + ThreadRemote + Send + Sync
{
}

#[async_trait]
pub trait MailCache {
    async fn get_mail_state(&self) -> Option<&GetState>;

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<()>;

    async fn get_mail_core(&self, id: &MailId) -> Result<Option<MailDataCore>>
    where
        Self: Sync,
    {
        let result = self.get_mails_core(&[id.clone()]).await?;

        if result.missing.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap().1))
        } else {
            Ok(None)
        }
    }

    async fn get_mails_core(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataCore>, Vec<MailId>>>;

    async fn get_mail_preview(&self, id: &MailId) -> Result<Option<MailDataPreview>>
    where
        Self: Sync,
    {
        let result = self.get_mails_preview(&[id.clone()]).await?;

        if result.missing.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap().1))
        } else {
            Ok(None)
        }
    }

    async fn get_mails_preview(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataPreview>, Vec<MailId>>>;

    async fn upsert_mails_core(&mut self, mails: Vec<(MailId, MailDataCore)>) -> Result<()>;

    async fn upsert_mails_preview(&mut self, mails: Vec<(MailId, MailDataPreview)>) -> Result<()>;

    async fn get_mail_text_body(&self, id: &MailId) -> Result<Option<MailDataTextBody>>
    where
        Self: Sync,
    {
        let result = self.get_mails_text_body(&[id.clone()]).await?;

        if !result.value.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap().1))
        } else {
            Ok(None)
        }
    }

    async fn get_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataTextBody>, Vec<MailId>>>;

    async fn upsert_mail_text_body(&mut self, id: &MailId, body: MailDataTextBody) -> Result<()>;

    async fn upsert_mails_text_body(
        &mut self,
        text_bodies: &[(MailId, MailDataTextBody)],
    ) -> Result<()>;

    async fn get_mail_html_body(&self, id: &MailId) -> Result<Option<MailDataHtmlBody>>
    where
        Self: Sync,
    {
        let result = self.get_mails_html_body(&[id.clone()]).await?;

        if !result.value.is_empty() {
            Ok(Some(result.value.into_iter().next().unwrap().1))
        } else {
            Ok(None)
        }
    }

    async fn get_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataHtmlBody>, Vec<MailId>>>;

    async fn upsert_mail_html_body(&mut self, id: &MailId, body: MailDataHtmlBody) -> Result<()>;

    async fn upsert_mails_html_body(
        &mut self,
        html_bodies: &[(MailId, MailDataHtmlBody)],
    ) -> Result<()>;

    async fn evict_mails(&mut self, mails: &[MailId]) -> Result<()>;
}

#[async_trait]
pub trait MailRemote {
    async fn fetch_mail_core(&self, id: MailId) -> Result<remote::GetOneResult<MailDataCore>>
    where
        Self: Sync,
    {
        let result = self.fetch_mails_core(&[id.clone()]).await?;

        Ok(remote::GetOneResult {
            value: result
                .values
                .into_iter()
                .next()
                .map(|(_id, data)| data)
                .expect("Id is valid"),
            state: result.state,
        })
    }

    async fn fetch_mails_core(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<HashMap<MailId, MailDataCore>, Vec<MailId>>>;

    async fn fetch_mail_preview(&self, id: MailId) -> Result<remote::GetOneResult<MailDataPreview>>
    where
        Self: Sync,
    {
        let result = self.fetch_mails_preview(&[id.clone()]).await?;

        Ok(remote::GetOneResult {
            value: result
                .values
                .into_iter()
                .next()
                .map(|(_id, data)| data)
                .expect("Id is valid"),
            state: result.state,
        })
    }

    async fn fetch_mails_preview(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<HashMap<MailId, MailDataPreview>, Vec<MailId>>>;

    async fn fetch_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<HashMap<MailId, MailDataTextBody>, Vec<MailId>>>;

    async fn fetch_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<HashMap<MailId, MailDataHtmlBody>, Vec<MailId>>>;

    async fn fetch_mail_updates(
        &self,
        cores: &[MailId],
        previews: &[MailId],
        text: &[MailId],
        html: &[MailId],
    ) -> Result<
        remote::GetOneResult<(
            Vec<(MailId, MailDataCore)>,
            Vec<(MailId, MailDataPreview)>,
            Vec<(MailId, MailDataTextBody)>,
            Vec<(MailId, MailDataHtmlBody)>,
        )>,
    >;

    // async async fn create_mail(
    //     &self,
    //     new: MailNew,
    //     since: GetState,
    // ) -> Result<remote::CreateResult<MailData>>;

    // async async fn update_mails(
    //     &self,
    //     updates: Vec<(MailData, MailUpdate)>,
    //     since: GetState,
    // ) -> Result<remote::UpdateResult<MailId, MailData>>;

    async fn destroy_mails(
        &self,
        ids: &[MailId],
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>>;

    async fn fetch_mail_changes(&self, since: &GetState)
    -> Result<remote::GetChangeResult<MailId>>;

    async fn fetch_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<remote::GetOneResult<MailDataTextBody>>
    where
        Self: Sync,
    {
        let result = self.fetch_mails_text_body(&[id.clone()]).await?;

        Ok(remote::GetOneResult {
            value: result
                .values
                .into_iter()
                .next()
                .map(|(_id, text_body)| text_body)
                .expect("Id is valid"),
            state: result.state,
        })
    }

    async fn fetch_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<remote::GetOneResult<MailDataHtmlBody>>
    where
        Self: Sync,
    {
        let result = self.fetch_mails_html_body(&[id.clone()]).await?;

        Ok(remote::GetOneResult {
            value: result
                .values
                .into_iter()
                .next()
                .map(|(_id, html_body)| html_body)
                .expect("MailId is valid"),
            state: result.state,
        })
    }
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

    async fn calculate_total_root_mails(&self, mailbox: &MailboxId) -> Result<Option<usize>>;

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
        calculate_total: bool,
    ) -> Result<remote::QueryResponse<remote::GetOneResult<Vec<(MailId, MailDataCore)>>>>;

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

    async fn get_mailbox(&self, id: &MailboxId) -> Result<Option<MailboxData>>
    where
        Self: Sync,
    {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.value.into_iter().next())
    }

    async fn get_all_mailboxes(&self) -> Result<Option<Vec<MailboxData>>>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetBatchResult<Vec<MailboxData>, Vec<MailboxId>>>;

    async fn get_mailbox_children(
        &self,
        parent: &ParentMailboxId,
    ) -> Result<Option<Vec<MailboxData>>>;

    async fn upsert_mailboxes(
        &mut self,
        mailboxes: Vec<MailboxData>,
        state: GetState,
    ) -> Result<()>;

    async fn evict_mailboxes(&mut self, ids: &[MailboxId], new_state: GetState) -> Result<()>;
}

#[async_trait]
pub trait MailboxRemote {
    async fn fetch_mailboxes_all(&self) -> Result<remote::GetOneResult<Vec<MailboxData>>>;

    async fn fetch_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailboxId>>;

    async fn create_mailbox(&self, new: MailboxNew) -> Result<remote::CreateResult<MailboxData>>;

    async fn update_mailboxes(
        &self,
        updates: Vec<(MailboxData, MailboxUpdate)>,
        since: &GetState,
    ) -> Result<remote::UpdateResult<MailboxId, MailboxData>>;

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
        self.upsert_threads(&[(id, mails)]).await
    }

    async fn upsert_threads(&mut self, threads: &[(ThreadId, Vec<MailId>)]) -> Result<()>;

    async fn evict_threads(&mut self, ids: &[ThreadId]) -> Result<()>;
}

#[async_trait]
pub trait ThreadRemote {
    async fn fetch_thread(
        &self,
        id: &ThreadId,
    ) -> Result<remote::GetOneResult<remote::GetOneResult<Vec<(MailId, MailDataCore)>>>>;

    async fn fetch_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<
        remote::GetBatchResult<
            remote::GetOneResult<HashMap<ThreadId, Vec<MailDataCore>>>,
            Vec<ThreadId>,
        >,
    >;

    async fn fetch_thread_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<ThreadId>>;
}
