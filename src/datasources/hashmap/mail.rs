#[derive(thiserror::Error, Debug, Clone)]
pub enum QueryError {
    #[error("")]
    NoRootMails,

    #[error("")]
    AnchorNotFound,
}

use crate::{
    datasources::{
        MailCache, MailDataSource,
        hashmap::{Error, HashMapDataSource},
        types::{Coverage, GetResult, GetState, QueryResponse, QueryState, QueryWindow, SetResult},
    },
    types::{
        MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailUpdate,
        MailboxId,
    },
};

impl MailDataSource for HashMapDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<GetResult<Vec<Option<MailData>>>, Self::Error> {
        todo!()
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<MailDataTextBody>, Self::Error> {
        todo!()
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<MailDataHtmlBody>, Self::Error> {
        todo!()
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Vec<MailDataAttachment>>, Self::Error> {
        todo!()
    }

    async fn create_mail(&self) -> Result<SetResult<MailData>, Self::Error> {
        todo!()
    }

    async fn update_mail(&self, update: MailUpdate) -> Result<SetResult<()>, Self::Error> {
        todo!()
    }

    async fn destroy_mail(&self, id: &MailId) -> Result<SetResult<()>, Self::Error> {
        todo!()
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow<MailId>,
    ) -> Result<QueryResponse<MailId>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let root_mails = inner
            .root_mails
            .get(mailbox)
            .ok_or(QueryError::NoRootMails)?;

        todo!();
    }
}

impl MailCache for HashMapDataSource {
    async fn upsert_mails(
        &self,
        mails: &[MailData],
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn evict_mails(&self, mails: &[MailId], new_state: GetState) -> Result<(), Self::Error> {
        todo!()
    }

    async fn upsert_query_mails(
        &self,
        mailbox: &MailboxId,
        ids: &[MailId],
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
