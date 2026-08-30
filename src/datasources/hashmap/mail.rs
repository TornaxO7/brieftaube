use crate::{
    datasources::{
        MailCache, MailDataSource,
        hashmap::{HashMapDataSource, utils::root_mails::RootMails},
        types::{GetResult, GetState, QueryResponse, QueryState, QueryWindow},
    },
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
};

impl MailDataSource for HashMapDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<GetResult<Vec<Option<MailData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let datas = ids.iter().map(|id| inner.mails.get(id).cloned()).collect();

        Ok(GetResult {
            value: datas,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataTextBody>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let body = inner.mail_text_body.get(id).cloned();

        Ok(GetResult {
            value: body,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<MailDataHtmlBody>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let body = inner.mail_html_body.get(id).cloned();

        Ok(GetResult {
            value: body,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<GetResult<Option<Vec<MailDataAttachment>>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let attachments = inner.mail_attachments.get(id).cloned();

        Ok(GetResult {
            value: attachments,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<QueryResponse<MailId>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let root_mails = inner
            .root_mails
            .get(mailbox)
            .expect("each mailbox has `RootMails`");

        let range = window.start..(window.start + window.limit);

        Ok(root_mails.query(range))
    }
}

impl MailCache for HashMapDataSource {
    async fn upsert_mails(
        &self,
        mails: Vec<MailData>,
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for mail in mails.into_iter() {
            let id = mail.id.clone();
            inner.mails.insert(id, mail);
        }

        inner.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_mails(&self, mails: &[MailId], new_state: GetState) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for id in mails {
            inner.mail_text_body.remove(id);
            inner.mail_html_body.remove(id);
            inner.mail_attachments.remove(id);

            if let Some(mail) = inner.mails.remove(id) {
                for mailbox in mail.mailbox_ids {
                    // removing leads to position changes, mail changes (in a thread) etc.
                    // => Just clear it.
                    inner
                        .root_mails
                        .entry(mailbox)
                        .and_modify(|root_mails| root_mails.flush());
                }
            }
        }

        inner.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn upsert_root_mails(
        &self,
        id: &MailboxId,
        start: usize,
        ids: Vec<MailId>,
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        inner
            .root_mails
            .entry(id.clone())
            .and_modify(|root_mails| root_mails.set(start, ids.clone(), new_state.clone()))
            .or_insert(RootMails::new(start, ids, new_state));

        Ok(())
    }
}
