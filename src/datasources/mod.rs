// mod database;
mod hashmap;
mod jmap;
pub mod types;

use crate::{
    datasources::types::{RemoteGetResult, RemoteQueryResponse},
    types::{
        MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailNew,
        MailUpdate, MailboxData, MailboxId, MailboxNew, MailboxUpdate, ThreadId,
    },
};
use types::{
    GetChangeResult, GetState, LocalGetResult, LocalQueryResponse, QueryChangeResult, QueryState,
    QueryWindow, RemoteSetResult,
};

pub trait BaseDataSource {
    type Error;
}

pub trait MailCache: BaseDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<LocalGetResult<Vec<Option<MailData>>>, Self::Error>;

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<LocalGetResult<Option<MailDataTextBody>>, Self::Error>;

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<LocalGetResult<Option<MailDataHtmlBody>>, Self::Error>;

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<LocalGetResult<Option<Vec<MailDataAttachment>>>, Self::Error>;

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<LocalQueryResponse<MailId>, Self::Error>;

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
    ) -> Result<RemoteGetResult<MailId, Vec<MailData>>, Self::Error>;

    async fn fetch_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, MailDataTextBody>, Self::Error>;

    async fn fetch_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, MailDataHtmlBody>, Self::Error>;

    async fn fetch_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, Vec<MailDataAttachment>>, Self::Error>;

    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<RemoteQueryResponse<MailId>, Self::Error>;

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<RemoteSetResult<MailData>, Self::Error>;

    // TODO: Allow batches
    async fn update_mail(
        &self,
        update: MailUpdate,
        since: GetState,
    ) -> Result<RemoteSetResult<()>, Self::Error>;

    async fn destroy_mail(
        &self,
        id: &MailId,
        since: GetState,
    ) -> Result<RemoteSetResult<()>, Self::Error>;

    async fn fetch_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<GetChangeResult<MailId>, Self::Error>;

    async fn fetch_root_mail_changes(
        &self,
        since: &QueryState,
    ) -> Result<QueryChangeResult<MailId>, Self::Error>;
}

pub trait MailboxDataSource: BaseDataSource {
    async fn get_mailbox(
        &self,
        id: &MailboxId,
    ) -> Result<LocalGetResult<Option<MailboxData>>, Self::Error> {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.map(|mailboxes| mailboxes.into_iter().next().flatten()))
    }

    async fn get_all_mailboxes(&self) -> Result<LocalGetResult<Vec<MailboxData>>, Self::Error>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<LocalGetResult<Vec<Option<MailboxData>>>, Self::Error>;
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
    async fn get_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<GetChangeResult<MailboxId>, Self::Error>;

    async fn create_mailbox(
        &self,
        new: MailboxNew,
    ) -> Result<RemoteSetResult<MailboxData>, Self::Error>;

    async fn update_mailbox(
        &self,
        update: MailboxUpdate,
    ) -> Result<RemoteSetResult<()>, Self::Error>;

    async fn destroy_mailbox(&self, id: &MailboxId) -> Result<RemoteSetResult<()>, Self::Error>;
}

pub trait ThreadDataSource: BaseDataSource {
    async fn get_thread(
        &self,
        id: &ThreadId,
    ) -> Result<LocalGetResult<Option<Vec<MailId>>>, Self::Error> {
        let threads = self.get_threads(&[id.clone()]).await?;
        Ok(threads.map(|mails| mails.into_iter().next().flatten()))
    }

    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<LocalGetResult<Vec<Option<Vec<MailId>>>>, Self::Error>;
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
    ) -> Result<GetChangeResult<MailId>, Self::Error>;
}
