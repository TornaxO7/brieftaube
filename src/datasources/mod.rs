// mod database;
mod hashmap;
mod jmap;
pub mod types;

use crate::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailUpdate,
    MailboxData, MailboxId, MailboxNew, MailboxUpdate, ThreadId,
};
use types::{
    GetChangeResult, GetResult, GetState, QueryChangeResult, QueryResponse, QueryState,
    QueryWindow, SetResult,
};

pub trait BaseDataSource {
    type Error;
}

pub trait MailDataSource: BaseDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<GetResult<Vec<Option<MailData>>>, Self::Error>;

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataTextBody>>, Self::Error>;

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataHtmlBody>>, Self::Error>;

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<Vec<MailDataAttachment>>>, Self::Error>;

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<QueryResponse<MailId>, Self::Error>;
}

pub trait MailCache: BaseDataSource {
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
    async fn create_mail(&self) -> Result<SetResult<MailData>, Self::Error>;

    async fn update_mail(&self, update: MailUpdate) -> Result<SetResult<()>, Self::Error>;

    async fn destroy_mail(&self, id: &MailId) -> Result<SetResult<()>, Self::Error>;

    async fn get_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<GetChangeResult<MailId>, Self::Error>;

    async fn get_root_mail_changes(
        &self,
        since: &QueryState,
    ) -> Result<QueryChangeResult<MailId>, Self::Error>;
}

pub trait MailboxDataSource: BaseDataSource {
    async fn get_mailbox(
        &self,
        id: &MailboxId,
    ) -> Result<GetResult<Option<MailboxData>>, Self::Error> {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.map(|mailboxes| mailboxes.into_iter().next().flatten()))
    }

    async fn get_all_mailboxes(&self) -> Result<GetResult<Vec<MailboxData>>, Self::Error>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<GetResult<Vec<Option<MailboxData>>>, Self::Error>;
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

    async fn create_mailbox(&self, new: MailboxNew) -> Result<SetResult<MailboxData>, Self::Error>;

    async fn update_mailbox(&self, update: MailboxUpdate) -> Result<SetResult<()>, Self::Error>;

    async fn destroy_mailbox(&self, id: &MailboxId) -> Result<SetResult<()>, Self::Error>;
}

pub trait ThreadDataSource: BaseDataSource {
    async fn get_thread(
        &self,
        id: &ThreadId,
    ) -> Result<GetResult<Option<Vec<MailId>>>, Self::Error> {
        let threads = self.get_threads(&[id.clone()]).await?;
        Ok(threads.map(|mails| mails.into_iter().next().flatten()))
    }

    async fn get_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<GetResult<Vec<Option<Vec<MailId>>>>, Self::Error>;
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
