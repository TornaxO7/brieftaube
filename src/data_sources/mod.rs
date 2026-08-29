use crate::backend::types::{
    MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailUpdate,
    MailboxData, MailboxId, ThreadId,
};

pub trait DataSource: DataSourceMail + DataSourceMailbox + DataSourceThread {}

pub trait DataSourceMail {
    type Error;

    async fn get_mails(&self, ids: &[MailId]) -> Result<Vec<MailData>, Self::Error>;

    async fn get_mail_text_body(&self, id: &MailId) -> Result<MailDataTextBody, Self::Error>;

    async fn get_mail_html_body(&self, id: &MailId) -> Result<MailDataHtmlBody, Self::Error>;

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Vec<MailDataAttachment>, Self::Error>;

    async fn create_mail(&self) -> Result<MailData, Self::Error>;

    async fn update_mail(&self, update: MailUpdate) -> Result<(), Self::Error>;

    async fn delete_mail(&self, id: &MailId) -> Result<(), Self::Error>;
}

pub trait DataSourceMailbox {
    async fn get_mailbox(&self, id: &MailboxId) -> MailboxData;
}

pub trait DataSourceThread {
    async fn get_thread(&self, id: &ThreadId) -> Vec<MailId>;
}
