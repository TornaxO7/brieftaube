// mod database;
mod hashmap;
mod jmap;
pub mod types;

use crate::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailNew, MailUpdate,
    MailboxData, MailboxId, MailboxNew, MailboxUpdate, ThreadId,
};
use types::{GetState, QueryState, QueryWindow, cache, remote};

pub trait BaseDataSource {
    type Error;
}

pub trait MailCache: BaseDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetResult<Vec<Option<MailData>>>, Self::Error>;

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<MailDataTextBody>>, Self::Error>;

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<MailDataHtmlBody>>, Self::Error>;

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<Vec<MailDataAttachment>>>, Self::Error>;

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<cache::QueryResponse<MailId>, Self::Error>;

    async fn upsert_mails(
        &self,
        mails: Vec<MailData>,
        new_state: GetState,
    ) -> Result<(), Self::Error>;

    async fn evict_mails(&self, mails: &[MailId], new_state: GetState) -> Result<(), Self::Error>;

    async fn upsert_root_mails(
        &self,
        mailbox: &MailboxId,
        start: usize,
        ids: Vec<MailId>,
        new_state: QueryState,
    ) -> Result<(), Self::Error>;
}

pub trait MailRemote: BaseDataSource {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, MailData>, Self::Error>;

    async fn fetch_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, MailDataTextBody>, Self::Error>;

    async fn fetch_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, MailDataHtmlBody>, Self::Error>;

    async fn fetch_mails_attachments(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, Vec<MailDataAttachment>>, Self::Error>;

    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<remote::QueryResponse<MailId>, Self::Error>;

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<remote::CreateResult<MailData>, Self::Error>;

    // TODO: Allow batches
    async fn update_mails(
        &self,
        updates: Vec<(MailData, MailUpdate)>,
        since: GetState,
    ) -> Result<remote::UpdateResult<MailId, MailData>, Self::Error>;

    async fn destroy_mails(
        &self,
        ids: Vec<MailId>,
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>, Self::Error>;

    async fn fetch_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailId>, Self::Error>;

    async fn fetch_root_mail_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
    ) -> Result<remote::QueryChangeResult<MailId>, Self::Error>;
}

pub trait MailboxDataSource: BaseDataSource {
    async fn get_mailbox(
        &self,
        id: &MailboxId,
    ) -> Result<cache::GetResult<Option<MailboxData>>, Self::Error> {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.map(|mailboxes| mailboxes.into_iter().next().flatten()))
    }

    async fn get_all_mailboxes(&self) -> Result<cache::GetResult<Vec<MailboxData>>, Self::Error>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetResult<Vec<Option<MailboxData>>>, Self::Error>;
}

pub trait MailboxCache: BaseDataSource {
    async fn upsert_mailboxes(
        &self,
        mailboxes: Vec<MailboxData>,
        state: GetState,
    ) -> Result<(), Self::Error>;

    async fn evict_mailboxes(
        &self,
        ids: &[MailboxId],
        new_state: GetState,
    ) -> Result<(), Self::Error>;
}

pub trait MailboxRemote: BaseDataSource {
    async fn fetch_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailboxId>, Self::Error>;

    async fn create_mailbox(
        &self,
        new: MailboxNew,
    ) -> Result<remote::CreateResult<MailboxData>, Self::Error>;

    async fn update_mailboxes(
        &self,
        updates: Vec<(MailboxData, MailboxUpdate)>,
    ) -> Result<remote::UpdateResult<MailboxId, MailboxData>, Self::Error>;

    async fn destroy_mailboxes(
        &self,
        ids: &[MailboxId],
        on_destroy_remove_emails: bool,
    ) -> Result<remote::DestroyResult<MailboxId>, Self::Error>;
}

pub trait ThreadDataSource: BaseDataSource {
    async fn get_thread(
        &self,
        id: &ThreadId,
    ) -> Result<cache::GetResult<Option<Vec<MailId>>>, Self::Error> {
        let threads = self.get_threads(&[id.clone()]).await?;
        Ok(threads.map(|mails| mails.into_iter().next().flatten()))
    }

    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<cache::GetResult<Vec<Option<Vec<MailId>>>>, Self::Error>;
}

pub trait ThreadCache: BaseDataSource {
    async fn upsert_thread(
        &self,
        id: &ThreadId,
        mails: &[MailId],
        new_state: GetState,
    ) -> Result<(), Self::Error>;

    async fn evict_thread(&self, id: &ThreadId, new_state: GetState) -> Result<(), Self::Error>;
}

pub trait ThreadRemote: BaseDataSource {
    async fn get_thread_changes(
        &self,
        id: &ThreadId,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailId>, Self::Error>;
}
