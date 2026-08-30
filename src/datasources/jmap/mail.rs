use super::Jmap;
use crate::{
    datasources::{
        MailDataSource,
        types::{GetResult, QueryResponse, QueryWindow},
    },
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
};

impl MailDataSource for Jmap {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<GetResult<Vec<Option<MailData>>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request
                .get_email()
                .ids(Some(ids))
                .properties(MailData::PROPERTIES);

            request.send_get_email().await?
        };

        let state = response.take_state();
        todo!()
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataTextBody>>, Self::Error> {
        todo!()
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataHtmlBody>>, Self::Error> {
        todo!()
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<Vec<MailDataAttachment>>>, Self::Error> {
        todo!()
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<QueryResponse<MailId>, Self::Error> {
        todo!()
    }
}
