// mod database;
pub mod hashmap;
pub mod jmap;
pub mod types;

use std::collections::HashSet;

use crate::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailNew, MailUpdate,
    MailboxData, MailboxId, MailboxNew, MailboxUpdate, ParentMailboxId, ThreadId,
};
use types::{GetState, QueryState, QueryWindow, cache, remote};

pub trait BaseDataSource {
    type Error: std::fmt::Debug;
}

pub trait Cache: BaseDataSource + MailCache + RootMailsCache + MailboxCache + ThreadCache {}
pub trait Remote:
    BaseDataSource + MailRemote + RootMailsRemote + MailboxRemote + ThreadRemote
{
}

pub trait MailCache: BaseDataSource {
    async fn get_mail_state(&self) -> Option<&GetState>;

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<(), Self::Error>;

    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<MailData>, Vec<MailId>>, Self::Error>;

    async fn upsert_mails(&mut self, mails: Vec<MailData>) -> Result<(), Self::Error>;

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataTextBody>, Self::Error>;

    async fn get_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<(MailId, MailDataTextBody)>, Vec<MailId>>, Self::Error>;

    async fn upsert_mail_text_body(
        &mut self,
        id: &MailId,
        body: MailDataTextBody,
    ) -> Result<(), Self::Error>;

    async fn upsert_mails_text_body(
        &mut self,
        text_bodies: &[(MailId, MailDataTextBody)],
    ) -> Result<(), Self::Error>;

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataHtmlBody>, Self::Error>;

    async fn get_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<(MailId, MailDataHtmlBody)>, Vec<MailId>>, Self::Error>;

    async fn upsert_mail_html_body(
        &mut self,
        id: &MailId,
        body: MailDataHtmlBody,
    ) -> Result<(), Self::Error>;

    async fn upsert_mails_html_body(
        &mut self,
        html_bodies: &[(MailId, MailDataHtmlBody)],
    ) -> Result<(), Self::Error>;

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Option<Vec<MailDataAttachment>>, Self::Error>;

    async fn get_mails_attachments(
        &self,
        ids: &[MailId],
    ) -> Result<
        cache::GetBatchResult<Vec<(MailId, Vec<MailDataAttachment>)>, Vec<MailId>>,
        Self::Error,
    >;

    async fn upsert_mail_attachments(
        &mut self,
        id: &MailId,
        attachments: Vec<MailDataAttachment>,
    ) -> Result<(), Self::Error>;

    async fn upsert_mails_attachments(
        &mut self,
        attachments: &[(MailId, Vec<MailDataAttachment>)],
    ) -> Result<(), Self::Error>;

    async fn evict_mails(&mut self, mails: &[MailId]) -> Result<(), Self::Error>;
}

pub trait MailRemote: BaseDataSource {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<Vec<MailData>, Vec<MailId>>, Self::Error>;

    async fn fetch_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<Vec<(MailId, MailDataTextBody)>, Vec<MailId>>, Self::Error>;

    async fn fetch_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetBatchResult<Vec<(MailId, MailDataHtmlBody)>, Vec<MailId>>, Self::Error>;

    async fn fetch_mails_attachments(
        &self,
        ids: &[MailId],
    ) -> Result<
        remote::GetBatchResult<Vec<(MailId, Vec<MailDataAttachment>)>, Vec<MailId>>,
        Self::Error,
    >;

    async fn fetch_mail_updates(
        &self,
        datas: &[MailId],
        text: &[MailId],
        html: &[MailId],
        attachments: &[MailId],
    ) -> Result<
        remote::GetOneResult<(
            Vec<MailData>,
            Vec<(MailId, MailDataTextBody)>,
            Vec<(MailId, MailDataHtmlBody)>,
            Vec<(MailId, Vec<MailDataAttachment>)>,
        )>,
        Self::Error,
    >;

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<remote::CreateResult<MailData>, Self::Error>;

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

    async fn fetch_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<remote::GetOneResult<MailDataTextBody>, Self::Error> {
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
    ) -> Result<remote::GetOneResult<MailDataHtmlBody>, Self::Error> {
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

    async fn fetch_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<remote::GetOneResult<Vec<MailDataAttachment>>, Self::Error> {
        let result = self.fetch_mails_attachments(&[id.clone()]).await?;

        Ok(remote::GetOneResult {
            value: result
                .values
                .into_iter()
                .next()
                .map(|(_id, attachments)| attachments)
                .expect("MailId is valid"),
            state: result.state,
        })
    }
}

pub trait RootMailsCache: MailCache {
    async fn get_root_mails_state(&self, mailbox: &MailboxId) -> Option<&QueryState>;

    async fn set_root_mails_state(
        &mut self,
        mailbox: &MailboxId,
        new_state: QueryState,
    ) -> Result<(), Self::Error>;

    async fn get_root_mails_last_id(&self, mailbox: &MailboxId) -> Option<MailId>;

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<Option<cache::QueryResponse<MailData>>, Self::Error>;

    async fn insert_root_mails<MailsWithIndex>(
        &mut self,
        mailbox: &MailboxId,
        root_mails: MailsWithIndex,
    ) -> Result<(), Self::Error>
    where
        MailsWithIndex: IntoIterator<Item = (MailId, usize)>;

    async fn evict_root_mails(
        &mut self,
        mailbox: &MailboxId,
        ids: HashSet<MailId>,
    ) -> Result<(), Self::Error>;
}

pub trait RootMailsRemote: MailRemote {
    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: &QueryWindow,
    ) -> Result<remote::QueryResponse<remote::GetOneResult<Vec<MailData>>>, Self::Error>;

    async fn fetch_root_mails_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
        up_to_id: Option<&MailId>,
    ) -> Result<remote::QueryChangeResult<MailId>, Self::Error>;
}

pub trait MailboxCache: BaseDataSource {
    async fn get_mailbox_state(&self) -> Option<&GetState>;

    async fn get_mailbox(&self, id: &MailboxId) -> Result<Option<MailboxData>, Self::Error> {
        let result = self.get_mailboxes(&[id.clone()]).await?;
        Ok(result.value.into_iter().next())
    }

    async fn get_all_mailboxes(&self) -> Result<Option<Vec<MailboxData>>, Self::Error>;

    async fn get_mailboxes(
        &self,
        ids: &[MailboxId],
    ) -> Result<cache::GetBatchResult<Vec<MailboxData>, Vec<MailboxId>>, Self::Error>;

    async fn get_mailbox_children(
        &self,
        parent: &ParentMailboxId,
    ) -> Result<Option<Vec<MailboxData>>, Self::Error>;

    async fn upsert_mailboxes(
        &mut self,
        mailboxes: Vec<MailboxData>,
        state: GetState,
    ) -> Result<(), Self::Error>;

    async fn evict_mailboxes(
        &mut self,
        ids: &[MailboxId],
        new_state: GetState,
    ) -> Result<(), Self::Error>;
}

pub trait MailboxRemote: BaseDataSource {
    async fn fetch_mailboxes_all(
        &self,
    ) -> Result<remote::GetOneResult<Vec<MailboxData>>, Self::Error>;

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
        since: &GetState,
    ) -> Result<remote::UpdateResult<MailboxId, MailboxData>, Self::Error>;

    async fn destroy_mailboxes(
        &self,
        ids: &[MailboxId],
        on_destroy_remove_emails: bool,
    ) -> Result<remote::DestroyResult<MailboxId>, Self::Error>;
}

pub trait ThreadCache: BaseDataSource {
    async fn get_thread_state(&self) -> Option<&GetState>;

    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Vec<MailData>>, Self::Error>;

    async fn upsert_thread(
        &mut self,
        id: &ThreadId,
        mails: &[MailData],
        new_get_mail_state: GetState,
        new_get_thread_state: GetState,
    ) -> Result<(), Self::Error>;

    async fn evict_thread(&mut self, id: &ThreadId, new_state: GetState)
    -> Result<(), Self::Error>;
}

pub trait ThreadRemote: BaseDataSource {
    async fn fetch_thread(
        &self,
        id: &ThreadId,
    ) -> Result<remote::GetOneResult<remote::GetOneResult<Vec<MailData>>>, Self::Error>;

    async fn fetch_thread_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<ThreadId>, Self::Error>;
}
